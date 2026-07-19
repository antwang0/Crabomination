//! Tests for recentN card batches 21-30 (merged from per-batch micro-files).

mod recent21 {
    use crabomination::card::Keyword;
    use crabomination::catalog;
    use crabomination::game::types::{Attack, AttackTarget};
    use crabomination::game::*;
    use crabomination::mana::Color;
    use crabomination::TurnStep;

    fn count_named(g: &GameState, controller: usize, name: &str) -> usize {
        g.battlefield.iter().filter(|c| c.controller == controller && c.definition.name == name).count()
    }

    /// Skyknight Vanguard makes a Soldier when it attacks.
    #[test]
    fn skyknight_vanguard_makes_soldier_on_attack() {
        let mut g = two_player_game();
        let sky = g.add_card_to_battlefield(0, catalog::skyknight_vanguard());
        g.clear_sickness(sky);
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        while g.step != TurnStep::DeclareAttackers {
            g.perform_action(GameAction::PassPriority).unwrap();
        }
        g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker: sky,
            target: AttackTarget::Player(1),
        }]))
        .expect("attack");
        drain_stack(&mut g);
        assert_eq!(count_named(&g, 0, "Soldier"), 1);
    }

    /// Aerial Boost pumps and grants flying.
    #[test]
    fn aerial_boost_pumps_and_flies() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let boost = g.add_card_to_hand(0, catalog::aerial_boost());
        g.players[0].mana_pool.add(Color::White, 1);
        g.players[0].mana_pool.add_colorless(1);
        g.active_player_idx = 0;
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        cast_at(&mut g, boost, Target::Permanent(bear));
        let cp = g.computed_permanent(bear).unwrap();
        assert_eq!((cp.power, cp.toughness), (4, 4));
        assert!(cp.keywords.contains(&Keyword::Flying));
    }

    /// Boots of Speed grants +1/+0 and haste when equipped.
    #[test]
    fn boots_of_speed_grants_haste() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let boots = g.add_card_to_battlefield(0, catalog::boots_of_speed());
        g.players[0].mana_pool.add_colorless(1);
        g.active_player_idx = 0;
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::Equip { equipment: boots, target: bear }).expect("equip");
        let cp = g.computed_permanent(bear).unwrap();
        assert_eq!(cp.power, 3, "+1/+0");
        assert!(cp.keywords.contains(&Keyword::Haste));
    }

    /// Ankle Biter has deathtouch.
    #[test]
    fn ankle_biter_has_deathtouch() {
        let mut g = two_player_game();
        let snake = g.add_card_to_battlefield(0, catalog::ankle_biter());
        assert!(g.computed_permanent(snake).unwrap().keywords.contains(&Keyword::Deathtouch));
    }

    /// Trick Shot deals 6 to a creature.
    #[test]
    fn trick_shot_deals_six() {
        let mut g = two_player_game();
        let foe = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4
        let ts = g.add_card_to_hand(0, catalog::trick_shot());
        g.players[0].mana_pool.add(Color::Red, 1);
        g.players[0].mana_pool.add_colorless(4);
        g.active_player_idx = 0;
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        cast_at(&mut g, ts, Target::Permanent(foe));
        assert!(g.battlefield_find(foe).is_none(), "6 damage kills the 4/4");
    }

    /// Patient Naturalist mills three and grabs a land.
    #[test]
    fn patient_naturalist_grabs_land() {
        let mut g = two_player_game();
        g.add_card_to_library(0, catalog::forest());
        g.add_card_to_library(0, catalog::grizzly_bears());
        g.add_card_to_library(0, catalog::grizzly_bears());
        let pn = g.add_card_to_battlefield(0, catalog::patient_naturalist());
        let hand = g.players[0].hand.len();
        g.fire_self_etb_triggers(pn, 0);
        drain_stack(&mut g);
        assert_eq!(g.players[0].hand.len(), hand + 1, "a land went to hand");
        assert!(g.players[0].hand.iter().any(|c| c.definition.name == "Forest"));
    }

    /// Plan the Heist draws three.
    #[test]
    fn plan_the_heist_draws_three() {
        let mut g = two_player_game();
        for _ in 0..4 {
            g.add_card_to_library(0, catalog::grizzly_bears());
        }
        let plan = g.add_card_to_hand(0, catalog::plan_the_heist());
        g.players[0].mana_pool.add(Color::Blue, 2);
        g.players[0].mana_pool.add_colorless(2);
        let hand = g.players[0].hand.len();
        g.active_player_idx = 0;
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        cast(&mut g, plan);
        assert_eq!(g.players[0].hand.len(), hand - 1 + 3, "−1 cast + 3 drawn");
    }

    /// Wanted Griffin leaves a Mercenary on death.
    #[test]
    fn wanted_griffin_dies_to_mercenary() {
        let mut g = two_player_game();
        let griffin = g.add_card_to_battlefield(0, catalog::wanted_griffin());
        let mut evs = g.remove_to_graveyard_with_triggers(griffin);
        evs.push(GameEvent::CreatureDied { card_id: griffin });
        g.dispatch_triggers_for_events(&evs);
        drain_stack(&mut g);
        assert_eq!(count_named(&g, 0, "Mercenary"), 1);
    }

    /// Sterling Hound surveils 2 on ETB (top two cards may go to the graveyard).
    #[test]
    fn sterling_hound_surveils() {
        let mut g = two_player_game();
        g.add_card_to_library(0, catalog::forest());
        g.add_card_to_library(0, catalog::forest());
        let hound = g.add_card_to_battlefield(0, catalog::sterling_hound());
        // AutoDecider keeps cards on top by default; just assert it resolves cleanly
        // and the library is intact (no panic, no card loss).
        let lib = g.players[0].library.len();
        g.fire_self_etb_triggers(hound, 0);
        drain_stack(&mut g);
        assert_eq!(g.players[0].library.len(), lib, "surveil kept both on top");
    }

    /// Hardbristle Bandit untaps when you commit a crime.
    #[test]
    fn hardbristle_untaps_on_crime() {
        let mut g = two_player_game();
        let bandit = g.add_card_to_battlefield(0, catalog::hardbristle_bandit());
        g.battlefield_find_mut(bandit).unwrap().tapped = true;
        // Commit a crime: cast Lava Spike at the opponent.
        let ls = g.add_card_to_hand(0, catalog::lava_spike());
        g.players[0].mana_pool.add(Color::Red, 1);
        g.active_player_idx = 0;
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        cast_at(&mut g, ls, Target::Player(1));
        assert!(!g.battlefield_find(bandit).unwrap().tapped, "untapped by the crime trigger");
    }

    /// Rumbling Rockslide deals damage equal to your land count.
    #[test]
    fn rumbling_rockslide_scales_with_lands() {
        let mut g = two_player_game();
        for _ in 0..4 {
            g.add_card_to_battlefield(0, catalog::forest());
        }
        let foe = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4
        let rr = g.add_card_to_hand(0, catalog::rumbling_rockslide());
        g.players[0].mana_pool.add(Color::Red, 1);
        g.players[0].mana_pool.add_colorless(3);
        g.active_player_idx = 0;
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        cast_at(&mut g, rr, Target::Permanent(foe));
        assert!(g.battlefield_find(foe).is_none(), "4 lands → 4 damage kills the 4/4");
    }
}

mod recent22 {
    use crabomination::catalog;
    use crabomination::game::types::{Attack, AttackTarget};
    use crabomination::game::*;
    use crabomination::mana::Color;
    use crabomination::TurnStep;

    fn attack_with(g: &mut GameState, attacker: CardId) {
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        while g.step != TurnStep::DeclareAttackers {
            g.perform_action(GameAction::PassPriority).unwrap();
        }
        g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker,
            target: AttackTarget::Player(1),
        }]))
        .expect("attack");
        drain_stack(g);
    }

    /// Jeong Jeong's firebending 1 adds {R} on attack, and that mana survives the
    /// step change into combat damage (it doesn't empty until end of combat).
    #[test]
    fn jeong_jeong_firebending_adds_red_that_survives_steps() {
        let mut g = two_player_game();
        let jj = g.add_card_to_battlefield(0, catalog::jeong_jeong_the_deserter());
        g.clear_sickness(jj);
        attack_with(&mut g, jj);
        assert_eq!(g.players[0].mana_pool.amount(Color::Red), 1, "firebending 1 added {{R}}");
        // Move out of declare-attackers (mana would normally empty between steps).
        g.perform_action(GameAction::PassPriority).unwrap();
        g.perform_action(GameAction::PassPriority).unwrap();
        assert!(
            g.players[0].mana_pool.amount(Color::Red) >= 1,
            "firebending mana persists across the step change in combat"
        );
    }

    /// Once combat ends the firebending mana is cleared (doesn't leak into the
    /// second main phase).
    #[test]
    fn firebending_mana_clears_after_combat() {
        let mut g = two_player_game();
        let jj = g.add_card_to_battlefield(0, catalog::jeong_jeong_the_deserter());
        g.clear_sickness(jj);
        attack_with(&mut g, jj);
        while g.step != TurnStep::PostCombatMain {
            g.perform_action(GameAction::PassPriority).unwrap();
        }
        assert_eq!(
            g.players[0].mana_pool.amount(Color::Red), 0,
            "firebending mana gone after combat"
        );
        assert_eq!(g.players[0].firebending_kept_red, 0, "kept-mana tracker reset");
    }

    /// Sozin's Comet grants firebending 5 to your creatures; a Grizzly Bears then
    /// makes {R}{R}{R}{R}{R} when it attacks.
    #[test]
    fn sozins_comet_grants_firebending() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.clear_sickness(bear);
        let comet = g.add_card_to_hand(0, catalog::sozins_comet());
        g.step = TurnStep::PreCombatMain;
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        g.players[0].mana_pool.add(Color::Red, 2);
        g.players[0].mana_pool.add_colorless(3);
        g.perform_action(GameAction::CastSpell {
            card_id: comet, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast Sozin's Comet");
        drain_stack(&mut g);
        attack_with(&mut g, bear);
        assert_eq!(g.players[0].mana_pool.amount(Color::Red), 5, "granted firebending 5 added {{R}}×5");
    }

    /// Sneak (CR 702.190): during the declare blockers step you may cast Donatello's
    /// Technique for {U} by returning an unblocked attacker you control to hand.
    #[test]
    fn donatello_sneak_returns_unblocked_attacker_for_cheap() {
        use crabomination::game::types::Attack;
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.clear_sickness(bear);
        let don = g.add_card_to_hand(0, catalog::donatellos_technique());
        for _ in 0..3 { g.add_card_to_library(0, catalog::forest()); }
        g.attacking = vec![Attack { attacker: bear, target: AttackTarget::Player(1) }];
        g.step = TurnStep::DeclareBlockers;
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        g.players[0].mana_pool.add(Color::Blue, 1); // only the {U} Sneak cost
        let hand_before = g.players[0].hand.len();
        g.perform_action(GameAction::CastSpellAlternative {
            card_id: don, pitch_card: None, target: None,
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("Sneak-cast Donatello's Technique for {U}");
        drain_stack(&mut g);
        assert!(g.players[0].hand.iter().any(|c| c.id == bear), "unblocked attacker returned to hand");
        // Drew 2 (+2), bear back (+1), Donatello left hand (-1) → net +2.
        assert_eq!(g.players[0].hand.len(), hand_before + 2, "drew two and got the attacker back");
        assert_eq!(g.players[0].mana_pool.amount(Color::Blue), 0, "only {{U}} was paid");
    }

    /// Ran and Shaw's firebending 2 adds {R}{R} on attack.
    #[test]
    fn ran_and_shaw_firebending_two() {
        let mut g = two_player_game();
        let rs = g.add_card_to_battlefield(0, catalog::ran_and_shaw());
        g.clear_sickness(rs);
        attack_with(&mut g, rs);
        assert_eq!(g.players[0].mana_pool.amount(Color::Red), 2, "firebending 2 added {{R}}{{R}}");
    }

    /// Jennika's Technique deals 2 damage to each creature.
    #[test]
    fn jennikas_technique_sweeps_two() {
        let mut g = two_player_game();
        let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2 dies
        let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2 dies
        let jt = g.add_card_to_hand(0, catalog::jennikas_technique());
        g.step = TurnStep::PreCombatMain;
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        g.players[0].mana_pool.add(Color::Red, 1);
        g.players[0].mana_pool.add_colorless(2);
        g.perform_action(GameAction::CastSpell {
            card_id: jt, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast Jennika's Technique");
        drain_stack(&mut g);
        assert!(g.battlefield_find(mine).is_none() && g.battlefield_find(theirs).is_none(),
            "both 2/2s died to 2 damage each");
    }

    fn cast_creature(g: &mut GameState, card: CardId) {
        g.step = TurnStep::PreCombatMain;
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        g.players[0].mana_pool.add(Color::Black, 3);
        g.players[0].mana_pool.add(Color::Red, 3);
        g.players[0].mana_pool.add_colorless(4);
        g.perform_action(GameAction::CastSpell {
            card_id: card, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast creature");
        drain_stack(g);
    }

    /// Bloodthirst 1 (CR 702.54): Bloodrage Vampire enters with a +1/+1 counter
    /// only if an opponent took damage this turn.
    #[test]
    fn bloodrage_vampire_bloodthirst_conditional() {
        use crabomination::card::CounterType;
        // No opponent damage → enters a vanilla 3/1.
        let mut g = two_player_game();
        let v1 = g.add_card_to_hand(0, catalog::bloodrage_vampire());
        cast_creature(&mut g, v1);
        assert_eq!(g.battlefield_find(v1).unwrap().counter_count(CounterType::PlusOnePlusOne), 0,
            "no bloodthirst without opponent damage");

        // Opponent took damage this turn → enters with one +1/+1 counter.
        let mut g = two_player_game();
        g.players[1].was_dealt_damage_this_turn = true;
        let v2 = g.add_card_to_hand(0, catalog::bloodrage_vampire());
        cast_creature(&mut g, v2);
        assert_eq!(g.battlefield_find(v2).unwrap().counter_count(CounterType::PlusOnePlusOne), 1,
            "bloodthirst 1 adds a counter");
    }

    /// Furyborn Hellkite's bloodthirst 6 adds six counters after opponent damage.
    #[test]
    fn furyborn_hellkite_bloodthirst_six() {
        use crabomination::card::CounterType;
        let mut g = two_player_game();
        g.players[1].was_dealt_damage_this_turn = true;
        let dragon = g.add_card_to_hand(0, catalog::furyborn_hellkite());
        g.step = TurnStep::PreCombatMain;
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        g.players[0].mana_pool.add(Color::Red, 3);
        g.players[0].mana_pool.add_colorless(4);
        g.perform_action(GameAction::CastSpell {
            card_id: dragon, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast Furyborn Hellkite");
        drain_stack(&mut g);
        assert_eq!(g.battlefield_find(dragon).unwrap().counter_count(CounterType::PlusOnePlusOne), 6,
            "bloodthirst 6 adds six counters");
    }

    /// The server view greys out Sneak's alt cast unless there's an unblocked
    /// attacker to return (return-to-hand feasibility gating).
    #[test]
    fn server_view_gates_sneak_on_unblocked_attacker() {
        use crabomination::game::types::Attack;
        use crabomination::net::HandCardView;
        let alt_available = |g: &GameState, name: &str| -> bool {
            let v = crabomination::server::view::project(g, 0);
            v.players[0].hand.iter().any(|h| matches!(h,
                HandCardView::Known(k) if k.name == name && k.alt_cost_available))
        };
        let mut g = two_player_game();
        g.add_card_to_hand(0, catalog::donatellos_technique());
        g.step = TurnStep::DeclareBlockers;
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        assert!(!alt_available(&g, "Donatello's Technique"),
            "no unblocked attacker → Sneak greyed out");

        // Add an unblocked attacker; now the Sneak alt cast is offered.
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.attacking = vec![Attack { attacker: bear, target: AttackTarget::Player(1) }];
        assert!(alt_available(&g, "Donatello's Technique"),
            "unblocked attacker present → Sneak available");
    }

    /// Sneak is only legal during your declare blockers step (CR 702.190a).
    #[test]
    fn sneak_rejected_outside_declare_blockers() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.clear_sickness(bear);
        let don = g.add_card_to_hand(0, catalog::donatellos_technique());
        g.step = TurnStep::PreCombatMain;
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        g.players[0].mana_pool.add(Color::Blue, 1);
        let res = g.perform_action(GameAction::CastSpellAlternative {
            card_id: don, pitch_card: None, target: None,
            additional_targets: vec![], mode: None, x_value: None,
        });
        assert!(res.is_err(), "Sneak only works during the declare blockers step");
    }

    /// Jeong Jeong's exhaust ability copies the next Lesson you cast this turn (and
    /// puts a +1/+1 counter on Jeong Jeong).
    #[test]
    fn jeong_jeong_copies_next_lesson() {
        use crabomination::card::CounterType;
        let mut g = two_player_game();
        let jj = g.add_card_to_battlefield(0, catalog::jeong_jeong_the_deserter());
        for _ in 0..8 { g.add_card_to_library(0, catalog::forest()); }
        g.step = TurnStep::PreCombatMain;
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        // Exhaust — {3}: +1/+1 counter + arm the next-Lesson copy.
        g.players[0].mana_pool.add_colorless(3);
        g.perform_action(GameAction::ActivateAbility {
            card_id: jj, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
        }).expect("activate exhaust");
        drain_stack(&mut g);
        assert_eq!(g.battlefield_find(jj).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
        // Cast Brilliant Plan (Lesson: scry 3, draw 3). The copy draws another 3.
        let hand_before = g.players[0].hand.len();
        let plan = g.add_card_to_hand(0, catalog::brilliant_plan());
        g.players[0].mana_pool.add(Color::Blue, 1);
        g.players[0].mana_pool.add_colorless(4);
        g.perform_action(GameAction::CastSpell {
            card_id: plan, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast Brilliant Plan");
        drain_stack(&mut g);
        // hand_before included the plan; it left the hand, then drew 3 + 3 = 6.
        assert_eq!(g.players[0].hand.len(), hand_before + 6, "original + copied Lesson drew six");
    }
}

mod recent23 {
    use crabomination::catalog;
    use crabomination::card::Keyword;
    use crabomination::game::types::{Attack, AttackTarget, Target};
    use crabomination::game::*;
    use crabomination::TurnStep;

    fn advance_to(g: &mut GameState, step: TurnStep) {
        while g.step != step {
            g.perform_action(GameAction::PassPriority).expect("pass priority");
        }
    }

    /// Doran makes every creature assign combat damage equal to its toughness: an
    /// unblocked 0/5 Doran deals 5 to the defending player.
    #[test]
    fn doran_attacks_for_toughness() {
        let mut g = two_player_game();
        let doran = g.add_card_to_battlefield(0, catalog::doran_the_siege_tower()); // 0/5
        g.clear_sickness(doran);
        advance_to(&mut g, TurnStep::DeclareAttackers);
        g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker: doran,
            target: AttackTarget::Player(1),
        }]))
        .expect("attack");
        drain_stack(&mut g);
        advance_to(&mut g, TurnStep::PostCombatMain);
        assert_eq!(g.players[1].life, 15, "0/5 Doran assigns 5 (toughness)");
    }

    /// Doran's substitution is unconditional even when power exceeds toughness: a
    /// 3/1 attacker assigns only 1.
    #[test]
    fn doran_caps_high_power_attacker_at_toughness() {
        let mut g = two_player_game();
        let doran = g.add_card_to_battlefield(0, catalog::doran_the_siege_tower());
        g.clear_sickness(doran);
        let bolt = g.add_card_to_battlefield(0, catalog::goblin_piker()); // 2/1
        g.clear_sickness(bolt);
        advance_to(&mut g, TurnStep::DeclareAttackers);
        g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker: bolt,
            target: AttackTarget::Player(1),
        }]))
        .expect("attack");
        drain_stack(&mut g);
        advance_to(&mut g, TurnStep::PostCombatMain);
        assert_eq!(g.players[1].life, 19, "2/1 under Doran assigns 1 (toughness)");
    }

    /// Tapestry Warden only affects your creatures whose toughness exceeds their
    /// power: a 1/4 Wall assigns 4, while a 2/1 you control assigns its normal 2.
    #[test]
    fn tapestry_warden_only_buffs_high_toughness() {
        let mut g = two_player_game();
        let warden = g.add_card_to_battlefield(0, catalog::tapestry_warden());
        g.clear_sickness(warden);
        // Warden itself is 3/4 (T>P) → assigns 4.
        advance_to(&mut g, TurnStep::DeclareAttackers);
        g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker: warden,
            target: AttackTarget::Player(1),
        }]))
        .expect("attack");
        drain_stack(&mut g);
        advance_to(&mut g, TurnStep::PostCombatMain);
        assert_eq!(g.players[1].life, 16, "3/4 Warden assigns 4 (toughness)");
    }

    /// Ancient Lumberknot reuses Tapestry Warden's static: a 1/4 it controls (T>P)
    /// assigns 4, attacking unblocked.
    #[test]
    fn ancient_lumberknot_buffs_high_toughness() {
        let mut g = two_player_game();
        let knot = g.add_card_to_battlefield(0, catalog::ancient_lumberknot()); // 1/4
        g.clear_sickness(knot);
        advance_to(&mut g, TurnStep::DeclareAttackers);
        g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker: knot,
            target: AttackTarget::Player(1),
        }]))
        .expect("attack");
        drain_stack(&mut g);
        advance_to(&mut g, TurnStep::PostCombatMain);
        assert_eq!(g.players[1].life, 16, "1/4 Lumberknot assigns 4 (toughness)");
    }

    /// Thrumming Hivepool's lord static grants double strike + haste to Slivers,
    /// and its Affinity for Slivers reduces its {6} cost by {1} per Sliver (so two
    /// Slivers let it cast for {4}).
    #[test]
    fn thrumming_hivepool_affinity_and_lord() {
        let mut g = two_player_game();
        let s1 = g.add_card_to_battlefield(0, catalog::muscle_sliver());
        g.add_card_to_battlefield(0, catalog::muscle_sliver());
        let pool = g.add_card_to_battlefield(0, catalog::thrumming_hivepool());
        assert!(
            g.computed_permanent(s1)
                .is_some_and(|c| c.keywords.contains(&Keyword::DoubleStrike)
                    && c.keywords.contains(&Keyword::Haste)),
            "Slivers gain double strike + haste"
        );
        // Affinity: {6} reduced by {1} per Sliver (2 on board) → {4} generic.
        let inst = g.battlefield.iter().find(|c| c.id == pool).unwrap().clone();
        let reduced = crabomination::game::actions::cost_reduction_for_spell(&g, 0, &inst, None);
        assert_eq!(reduced, 2, "Affinity for Slivers gives {{2}} off with two Slivers");
    }

    /// Bill the Pony enters with two Food and can sacrifice one to grant the
    /// toughness-damage keyword to a target creature you control until end of turn.
    #[test]
    fn bill_the_pony_etb_food_and_grant() {
        let mut g = two_player_game();
        let bill = g.move_card_to_battlefield_for_test(0, catalog::bill_the_pony());
        g.clear_sickness(bill);
        drain_stack(&mut g);
        let foods = g
            .battlefield
            .iter()
            .filter(|c| c.controller == 0 && c.is_token)
            .count();
        assert_eq!(foods, 2, "ETB makes two Food tokens");

        // Grant the keyword to Bill (a 1/4) by sacrificing a Food.
        g.perform_action(GameAction::ActivateAbility {
            card_id: bill,
            ability_index: 0,
            target: Some(Target::Permanent(bill)),
            additional_targets: Vec::new(),
            x_value: None,
        })
        .expect("activate sac-a-Food grant");
        drain_stack(&mut g);
        assert!(
            g.computed_permanent(bill)
                .is_some_and(|c| c.keywords.contains(&Keyword::AssignsCombatDamageByToughness)),
            "Bill now assigns combat damage by toughness"
        );
        let foods_after = g
            .battlefield
            .iter()
            .filter(|c| c.controller == 0 && c.is_token)
            .count();
        assert_eq!(foods_after, 1, "one Food sacrificed");
    }

    /// Bedhead Beastie is a 5/6 with menace and Mountaincycling {2}.
    #[test]
    fn bedhead_beastie_keywords() {
        let d = catalog::bedhead_beastie();
        assert_eq!((d.power, d.toughness), (5, 6));
        assert!(d.keywords.contains(&Keyword::Menace));
        assert!(d.keywords.iter().any(|k| matches!(k, Keyword::Typecycling(_))));
    }

    /// Daggermaw Megalodon is a 5/7 with vigilance and Islandcycling {2}.
    #[test]
    fn daggermaw_megalodon_keywords() {
        let d = catalog::daggermaw_megalodon();
        assert_eq!((d.power, d.toughness), (5, 7));
        assert!(d.keywords.contains(&Keyword::Vigilance));
        assert!(d.keywords.iter().any(|k| matches!(k, Keyword::Typecycling(_))));
    }

    /// Boilerbilges Ripper sacrifices another creature on ETB to deal 2 to any
    /// target (auto-decider sacrifices the fodder and pings the opponent).
    #[test]
    fn boilerbilges_ripper_sac_pings() {
        use crabomination::decision::{DecisionAnswer, ScriptedDecider};
        let mut g = two_player_game();
        let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
        g.move_card_to_battlefield_for_test(0, catalog::boilerbilges_ripper());
        drain_stack(&mut g);
        assert!(!g.battlefield.iter().any(|c| c.id == fodder), "fodder sacrificed");
        assert_eq!(g.players[1].life, 18, "dealt 2 to opponent");
    }

    /// Bashful Beastie manifests dread when it dies (a face-down 2/2 enters).
    #[test]
    fn bashful_beastie_dies_manifest_dread() {
        let mut g = two_player_game();
        let beastie = g.add_card_to_battlefield(0, catalog::bashful_beastie());
        // Seed library so manifest dread has cards to look at.
        g.add_card_to_library(0, catalog::grizzly_bears());
        g.add_card_to_library(0, catalog::grizzly_bears());
        let mut evs = g.remove_to_graveyard_with_triggers(beastie);
        evs.push(GameEvent::CreatureDied { card_id: beastie });
        g.dispatch_triggers_for_events(&evs);
        drain_stack(&mut g);
        assert!(
            g.battlefield.iter().any(|c| c.controller == 0 && c.face_down),
            "manifest dread put a face-down creature onto the battlefield"
        );
    }

    /// Bear Trap has flash and can sacrifice itself to deal 3 to a creature.
    #[test]
    fn bear_trap_sac_burns_creature() {
        let mut g = two_player_game();
        let trap = g.add_card_to_battlefield(0, catalog::bear_trap());
        let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g.players[0].mana_pool.add_colorless(3);
        g.perform_action(GameAction::ActivateAbility {
            card_id: trap,
            ability_index: 0,
            target: Some(Target::Permanent(bear)),
            additional_targets: Vec::new(),
            x_value: None,
        })
        .expect("activate Bear Trap");
        drain_stack(&mut g);
        assert!(!g.battlefield.iter().any(|c| c.id == bear), "2/2 took 3 and died");
        assert!(!g.battlefield.iter().any(|c| c.id == trap), "Bear Trap sacrificed");
    }

    /// Frantic Strength gives the enchanted creature +2/+2 and trample.
    #[test]
    fn frantic_strength_pumps_and_tramples() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
        let aura = g.add_card_to_hand(0, catalog::frantic_strength());
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g.players[0].mana_pool.add(crabomination::mana::Color::Green, 1);
        g.players[0].mana_pool.add_colorless(2);
        g.perform_action(GameAction::CastSpell {
            card_id: aura,
            target: Some(Target::Permanent(bear)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .expect("cast Frantic Strength");
        drain_stack(&mut g);
        let cp = g.computed_permanent(bear).unwrap();
        assert_eq!((cp.power, cp.toughness), (4, 4), "+2/+2");
        assert!(cp.keywords.contains(&Keyword::Trample), "granted trample");
    }

    /// Most Valuable Slayer's attack trigger gives an attacking creature +1/+0 and
    /// first strike.
    #[test]
    fn most_valuable_slayer_pumps_attacker() {
        let mut g = two_player_game();
        let slayer = g.add_card_to_battlefield(0, catalog::most_valuable_slayer()); // 2/4
        g.clear_sickness(slayer);
        advance_to(&mut g, TurnStep::DeclareAttackers);
        g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker: slayer,
            target: AttackTarget::Player(1),
        }]))
        .expect("attack");
        drain_stack(&mut g);
        let cp = g.computed_permanent(slayer).unwrap();
        assert_eq!(cp.power, 3, "attack trigger pumped +1/+0");
        assert!(cp.keywords.contains(&Keyword::FirstStrike), "gained first strike");
    }

    /// Twist Reality's first mode counters a spell on the stack.
    #[test]
    fn twist_reality_counters_spell() {
        let mut g = two_player_game();
        // Opponent casts a spell.
        let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
        g.players[1].mana_pool.add(crabomination::mana::Color::Red, 1);
        g.priority.player_with_priority = 1;
        g.step = TurnStep::PreCombatMain;
        g.perform_action(GameAction::CastSpell {
            card_id: bolt, target: Some(Target::Player(0)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("opponent casts bolt");
        // Counter it with Twist Reality (mode 0).
        let twist = g.add_card_to_hand(0, catalog::twist_reality());
        g.players[0].mana_pool.add(crabomination::mana::Color::Blue, 2);
        g.players[0].mana_pool.add_colorless(1);
        g.perform_action(GameAction::CastSpell {
            card_id: twist,
            target: Some(Target::Permanent(bolt)),
            additional_targets: vec![],
            mode: Some(0),
            x_value: None,
        }).expect("cast Twist Reality countering");
        drain_stack(&mut g);
        assert_eq!(g.players[0].life, 20, "bolt was countered (no damage)");
        assert!(g.players[1].graveyard.iter().any(|c| c.id == bolt), "countered bolt hits graveyard");
    }

    /// Vengeful Possession steals a creature until end of turn and untaps it.
    #[test]
    fn vengeful_possession_steals_creature() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        g.battlefield.iter_mut().find(|c| c.id == bear).unwrap().tapped = true;
        let spell = g.add_card_to_hand(0, catalog::vengeful_possession());
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g.players[0].mana_pool.add(crabomination::mana::Color::Red, 1);
        g.players[0].mana_pool.add_colorless(2);
        g.perform_action(GameAction::CastSpell {
            card_id: spell,
            target: Some(Target::Permanent(bear)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        }).expect("cast Vengeful Possession");
        drain_stack(&mut g);
        let cp = g.computed_permanent(bear).unwrap();
        assert_eq!(cp.controller, 0, "gained control of the creature");
        assert!(!g.battlefield.iter().find(|c| c.id == bear).unwrap().tapped, "untapped");
        assert!(cp.keywords.contains(&Keyword::Haste), "gained haste");
    }

    /// Unstoppable Plan untaps your nonland permanents at your end step.
    #[test]
    fn unstoppable_plan_untaps_at_end_step() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::unstoppable_plan());
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.battlefield.iter_mut().find(|c| c.id == bear).unwrap().tapped = true;
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        advance_to(&mut g, TurnStep::End);
        drain_stack(&mut g);
        assert!(!g.battlefield.iter().find(|c| c.id == bear).unwrap().tapped, "untapped at end step");
    }

    /// Gearseeker Serpent's Affinity for artifacts discounts its generic cost.
    #[test]
    fn gearseeker_serpent_affinity_for_artifacts() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::sol_ring());
        g.add_card_to_battlefield(0, catalog::sol_ring());
        g.add_card_to_battlefield(0, catalog::sol_ring());
        let serp = g.add_card_to_battlefield(0, catalog::gearseeker_serpent());
        let inst = g.battlefield.iter().find(|c| c.id == serp).unwrap().clone();
        let reduced = crabomination::game::actions::cost_reduction_for_spell(&g, 0, &inst, None);
        assert_eq!(reduced, 3, "three artifacts give {{3}} off");
    }

    /// Aetherjacket sacrifices itself to destroy another artifact.
    #[test]
    fn aetherjacket_sacs_to_destroy_artifact() {
        let mut g = two_player_game();
        let jacket = g.add_card_to_battlefield(0, catalog::aetherjacket());
        g.clear_sickness(jacket);
        let target = g.add_card_to_battlefield(1, catalog::sol_ring());
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g.players[0].mana_pool.add_colorless(2);
        g.perform_action(GameAction::ActivateAbility {
            card_id: jacket,
            ability_index: 0,
            target: Some(Target::Permanent(target)),
            additional_targets: Vec::new(),
            x_value: None,
        })
        .expect("activate Aetherjacket");
        drain_stack(&mut g);
        assert!(!g.battlefield.iter().any(|c| c.id == target), "target artifact destroyed");
        assert!(!g.battlefield.iter().any(|c| c.id == jacket), "Aetherjacket sacrificed");
    }

    /// Dynamite Diver deals 1 to any target when it dies.
    #[test]
    fn dynamite_diver_dies_pings() {
        let mut g = two_player_game();
        let diver = g.add_card_to_battlefield(0, catalog::dynamite_diver());
        let mut evs = g.remove_to_graveyard_with_triggers(diver);
        evs.push(GameEvent::CreatureDied { card_id: diver });
        g.dispatch_triggers_for_events(&evs);
        drain_stack(&mut g);
        assert_eq!(g.players[1].life, 19, "dies trigger pinged the opponent for 1");
    }

    /// Gas Guzzler starts your engines, enters tapped, and its max-speed ability is
    /// gated until speed 4.
    #[test]
    fn gas_guzzler_starts_engines_enters_tapped() {
        let mut g = two_player_game();
        let guz = g.move_card_to_battlefield_for_test(0, catalog::gas_guzzler());
        assert!(g.battlefield.iter().find(|c| c.id == guz).unwrap().tapped, "enters tapped");
        assert_eq!(g.players[0].speed, 1, "Start your engines! sets speed to 1");
    }

    /// Chitin Gravestalker's graveyard affinity discounts {1} per artifact/creature
    /// card in your graveyard.
    #[test]
    fn chitin_gravestalker_graveyard_affinity() {
        let mut g = two_player_game();
        g.add_card_to_graveyard(0, catalog::grizzly_bears()); // creature
        g.add_card_to_graveyard(0, catalog::sol_ring()); // artifact
        g.add_card_to_graveyard(0, catalog::lightning_bolt()); // neither — no discount
        let inst = crabomination::card::CardInstance::new(
            crabomination::card::CardId(9999),
            catalog::chitin_gravestalker(),
            0,
        );
        let reduced = crabomination::game::actions::cost_reduction_for_spell(&g, 0, &inst, None);
        assert_eq!(reduced, 2, "two matching gy cards give {{2}} off");
    }

    /// Unnerving Grasp bounces a target permanent and manifests dread.
    #[test]
    fn unnerving_grasp_bounces_and_manifests() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        g.add_card_to_library(0, catalog::grizzly_bears());
        g.add_card_to_library(0, catalog::grizzly_bears());
        let spell = g.add_card_to_hand(0, catalog::unnerving_grasp());
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g.players[0].mana_pool.add(crabomination::mana::Color::Blue, 1);
        g.players[0].mana_pool.add_colorless(2);
        g.perform_action(GameAction::CastSpell {
            card_id: spell, target: Some(Target::Permanent(bear)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast Unnerving Grasp");
        drain_stack(&mut g);
        assert!(g.players[1].hand.iter().any(|c| c.id == bear), "bear returned to hand");
        assert!(g.battlefield.iter().any(|c| c.controller == 0 && c.face_down),
            "manifest dread put a face-down creature onto the battlefield");
    }

    /// Fanged Flames deals 4 and exiles the creature instead of letting it die.
    #[test]
    fn fanged_flames_exiles_on_death() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
        let spell = g.add_card_to_hand(0, catalog::fanged_flames());
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g.players[0].mana_pool.add(crabomination::mana::Color::Red, 1);
        g.players[0].mana_pool.add_colorless(1);
        g.perform_action(GameAction::CastSpell {
            card_id: spell, target: Some(Target::Permanent(bear)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast Fanged Flames");
        drain_stack(&mut g);
        assert!(g.exile.iter().any(|c| c.id == bear), "lethal creature was exiled, not killed");
        assert!(!g.players[1].graveyard.iter().any(|c| c.id == bear), "not in graveyard");
    }

    /// Splitskin Doll draws, then discards when you control no other small creature.
    #[test]
    fn splitskin_doll_discards_without_small_creature() {
        let mut g = two_player_game();
        g.add_card_to_library(0, catalog::grizzly_bears());
        g.add_card_to_hand(0, catalog::grizzly_bears()); // a card to discard
        let hand_before = g.players[0].hand.len();
        g.move_card_to_battlefield_for_test(0, catalog::splitskin_doll());
        drain_stack(&mut g);
        // Drew 1, then discarded 1 (no other power-≤2 creature) → net hand unchanged.
        assert_eq!(g.players[0].hand.len(), hand_before, "draw then discard nets zero");
    }

    /// Skittering Surveyor fetches a basic land to hand on ETB.
    #[test]
    fn skittering_surveyor_fetches_land() {
        use crabomination::decision::{DecisionAnswer, ScriptedDecider};
        let mut g = two_player_game();
        let forest = g.add_card_to_library(0, catalog::forest());
        g.decider = Box::new(ScriptedDecider::new([
            DecisionAnswer::Bool(true),
            DecisionAnswer::Search(Some(forest)),
        ]));
        g.move_card_to_battlefield_for_test(0, catalog::skittering_surveyor());
        drain_stack(&mut g);
        assert!(g.players[0].hand.iter().any(|c| c.id == forest), "fetched the Forest to hand");
    }

    /// Agonasaur Rex's cycle trigger puts two +1/+1 counters on a creature and
    /// grants it trample + indestructible.
    #[test]
    fn agonasaur_rex_cycle_buffs_creature() {
        use crabomination::card::{CounterType, Keyword};
        use crabomination::decision::{DecisionAnswer, ScriptedDecider};
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let rex = g.add_card_to_hand(0, catalog::agonasaur_rex());
        g.add_card_to_library(0, catalog::grizzly_bears());
        g.players[0].mana_pool.add(crabomination::mana::Color::Green, 1);
        g.players[0].mana_pool.add_colorless(2);
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        // Target the bear with the reflexive cycle trigger.
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Target(Target::Permanent(bear))]));
        g.perform_action(GameAction::Cycle { card_id: rex, x_value: None }).expect("cycle Agonasaur Rex");
        drain_stack(&mut g);
        let cp = g.computed_permanent(bear).unwrap();
        assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne), 2,
            "two +1/+1 counters");
        assert!(cp.keywords.contains(&Keyword::Trample) && cp.keywords.contains(&Keyword::Indestructible),
            "granted trample + indestructible");
    }

    /// Marketwatch Phantom gains flying when another small creature you control
    /// enters.
    #[test]
    fn marketwatch_phantom_gains_flying() {
        use crabomination::card::Keyword;
        let mut g = two_player_game();
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        let phantom = g.add_card_to_battlefield(0, catalog::marketwatch_phantom());
        assert!(!g.computed_permanent(phantom).unwrap().keywords.contains(&Keyword::Flying),
            "no flying yet");
        // Cast a 2/1 (power ≤2) through the real ETB funnel so the trigger fires.
        let piker = g.add_card_to_hand(0, catalog::goblin_piker());
        g.players[0].mana_pool.add(crabomination::mana::Color::Red, 1);
        g.players[0].mana_pool.add_colorless(1);
        g.perform_action(GameAction::CastSpell {
            card_id: piker, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast a small creature");
        drain_stack(&mut g);
        assert!(g.computed_permanent(phantom).unwrap().keywords.contains(&Keyword::Flying),
            "gained flying when a small creature entered");
    }
}

mod recent24 {
    use crabomination::catalog;
    use crabomination::card::{CardType, CounterType, CreatureType, Keyword, Subtypes, TokenDefinition};
    use crabomination::game::types::{Attack, AttackTarget, Target};
    use crabomination::game::*;
    use crabomination::mana::Color;
    use crabomination::TurnStep;

    fn count_named(g: &GameState, controller: usize, name: &str) -> usize {
        g.battlefield
            .iter()
            .filter(|c| c.controller == controller && c.definition.name == name)
            .count()
    }

    fn advance_to(g: &mut GameState, step: TurnStep) {
        while g.step != step {
            g.perform_action(GameAction::PassPriority).expect("pass priority");
        }
    }

    /// Stand a player at PreCombatMain with priority and a full mana pool.
    fn ready(g: &mut GameState) {
        g.active_player_idx = 0;
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        for _ in 0..10 {
            g.players[0].mana_pool.add_colorless(1);
        }
        for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
            g.players[0].mana_pool.add(c, 4);
        }
    }

    /// Bounce Off returns a Vehicle to its owner's hand.
    #[test]
    fn bounce_off_returns_vehicle() {
        let mut g = two_player_game();
        let veh = g.add_card_to_battlefield(1, catalog::air_response_unit());
        let bo = g.add_card_to_hand(0, catalog::bounce_off());
        ready(&mut g);
        cast_at(&mut g, bo, Target::Permanent(veh));
        assert!(g.battlefield_find(veh).is_none(), "Vehicle bounced");
        assert_eq!(g.players[1].hand.len(), 1, "back in owner's hand");
    }

    /// Bestow Greatness pumps +4/+4 and grants trample.
    #[test]
    fn bestow_greatness_pumps_and_tramples() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
        let bg = g.add_card_to_hand(0, catalog::bestow_greatness());
        ready(&mut g);
        cast_at(&mut g, bg, Target::Permanent(bear));
        let cp = g.computed_permanent(bear).unwrap();
        assert_eq!((cp.power, cp.toughness), (6, 6));
        assert!(cp.keywords.contains(&Keyword::Trample));
    }

    /// Cult Healer's Eerie trigger fires when an enchantment (a Room) enters,
    /// granting it lifelink until end of turn.
    #[test]
    fn cult_healer_eerie_enchantment_enters() {
        let mut g = two_player_game();
        let healer = g.add_card_to_battlefield(0, catalog::cult_healer());
        let room = g.add_card_to_hand(0, catalog::unholy_annex_ritual_chamber());
        ready(&mut g);
        g.perform_action(GameAction::CastRoomDoor { card_id: room, right: false })
            .expect("cast Room (enchantment enters)");
        drain_stack(&mut g);
        assert!(
            g.computed_permanent(healer).unwrap().keywords.contains(&Keyword::Lifelink),
            "Cult Healer gained lifelink from Eerie",
        );
    }

    /// Balemurk Leech's Eerie trigger fires on fully unlocking a Room (both
    /// doors): the opponent loses 1 life.
    #[test]
    fn balemurk_leech_eerie_room_fully_unlocked() {
        let mut g = two_player_game();
        let room = g.add_card_to_hand(0, catalog::unholy_annex_ritual_chamber());
        ready(&mut g);
        // Open the right door first (no Leech yet → enchantment-enters is a no-op).
        g.perform_action(GameAction::CastRoomDoor { card_id: room, right: true })
            .expect("cast right door");
        drain_stack(&mut g);
        g.add_card_to_battlefield(0, catalog::balemurk_leech());
        let foe_life = g.players[1].life;
        // Unlock the left door → room fully unlocked → only the Eerie trigger.
        g.perform_action(GameAction::UnlockRoomDoor { card_id: room, right: false })
            .expect("unlock left door");
        drain_stack(&mut g);
        assert_eq!(g.battlefield_find(room).unwrap().unlocked_doors, 0b11, "fully unlocked");
        assert_eq!(g.players[1].life, foe_life - 1, "opponent lost 1 from Eerie");
    }

    /// Unwilling Vessel mints an X/X flying Spirit on death where X is its
    /// possession counters (LKI counter read, CR 603.10).
    #[test]
    fn unwilling_vessel_dies_mints_spirit() {
        let mut g = two_player_game();
        let uv = g.add_card_to_battlefield(0, catalog::unwilling_vessel());
        {
            let c = g.battlefield_find_mut(uv).unwrap();
            c.add_counters(CounterType::Possession, 2);
            c.damage = 2; // lethal vs 2 toughness
        }
        g.priority.player_with_priority = 0;
        g.check_state_based_actions();
        drain_stack(&mut g);
        let spirit = g
            .battlefield
            .iter()
            .find(|c| c.is_token && c.definition.name == "Spirit")
            .expect("Spirit token minted");
        assert_eq!((spirit.power(), spirit.toughness()), (2, 2), "X/X from 2 possession counters");
        assert!(spirit.definition.keywords.contains(&Keyword::Flying), "Spirit flies");
    }

    /// Gremlin Tamer's Eerie trigger mints a Gremlin when an enchantment enters.
    #[test]
    fn gremlin_tamer_eerie_makes_gremlin() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::gremlin_tamer());
        let room = g.add_card_to_hand(0, catalog::unholy_annex_ritual_chamber());
        ready(&mut g);
        g.perform_action(GameAction::CastRoomDoor { card_id: room, right: false })
            .expect("cast Room");
        drain_stack(&mut g);
        assert_eq!(count_named(&g, 0, "Gremlin"), 1, "Eerie minted a Gremlin");
    }

    /// Erratic Apparition's Eerie trigger pumps it +1/+1.
    #[test]
    fn erratic_apparition_eerie_pumps() {
        let mut g = two_player_game();
        let ea = g.add_card_to_battlefield(0, catalog::erratic_apparition());
        let room = g.add_card_to_hand(0, catalog::unholy_annex_ritual_chamber());
        ready(&mut g);
        g.perform_action(GameAction::CastRoomDoor { card_id: room, right: false })
            .expect("cast Room");
        drain_stack(&mut g);
        let cp = g.computed_permanent(ea).unwrap();
        assert_eq!((cp.power, cp.toughness), (2, 4), "+1/+1 from Eerie");
    }

    /// Commune with Evil puts one of the top four into hand, the rest into the
    /// graveyard, and gains 3 life.
    #[test]
    fn commune_with_evil_digs_and_gains() {
        let mut g = two_player_game();
        for _ in 0..4 {
            g.add_card_to_library(0, catalog::grizzly_bears());
        }
        let cwe = g.add_card_to_hand(0, catalog::commune_with_evil());
        ready(&mut g);
        let (hand, life) = (g.players[0].hand.len(), g.players[0].life);
        g.perform_action(GameAction::CastSpell {
            card_id: cwe, target: None, additional_targets: vec![], mode: None, x_value: None,
        })
        .expect("cast Commune with Evil");
        drain_stack(&mut g);
        assert_eq!(g.players[0].hand.len(), hand - 1 + 1, "one card to hand (spell left hand)");
        assert_eq!(g.players[0].graveyard.iter().filter(|c| c.definition.name == "Grizzly Bears").count(), 3, "rest to graveyard");
        assert_eq!(g.players[0].life, life + 3, "gained 3 life");
    }

    /// Acrobatic Cheerleader's Survival gives it a flying counter once it's tapped.
    #[test]
    fn acrobatic_cheerleader_survival_flies() {
        let mut g = two_player_game();
        let ac = g.add_card_to_battlefield(0, catalog::acrobatic_cheerleader());
        g.battlefield_find_mut(ac).unwrap().tapped = true;
        g.active_player_idx = 0;
        g.fire_step_triggers(TurnStep::PostCombatMain);
        drain_stack(&mut g);
        assert!(
            g.computed_permanent(ac).unwrap().keywords.contains(&Keyword::Flying),
            "gained flying from Survival",
        );
    }

    /// Clockwork Percussionist's death exiles the top card and grants a may-play.
    #[test]
    fn clockwork_percussionist_dies_impulse() {
        let mut g = two_player_game();
        let top = g.add_card_to_library(0, catalog::grizzly_bears());
        let cp = g.add_card_to_battlefield(0, catalog::clockwork_percussionist());
        g.battlefield_find_mut(cp).unwrap().damage = 1; // lethal vs 1 toughness
        g.priority.player_with_priority = 0;
        g.check_state_based_actions();
        drain_stack(&mut g);
        assert!(g.exile.iter().any(|c| c.id == top), "top card exiled with may-play");
    }

    /// Diversion Specialist sacrifices a creature to impulse the top card.
    #[test]
    fn diversion_specialist_sac_impulses() {
        let mut g = two_player_game();
        let top = g.add_card_to_library(0, catalog::grizzly_bears());
        g.add_card_to_battlefield(0, catalog::diversion_specialist());
        let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        ready(&mut g);
        let ds = g.battlefield.iter().find(|c| c.definition.name == "Diversion Specialist").unwrap().id;
        g.perform_action(GameAction::ActivateAbility {
            card_id: ds, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
        })
        .expect("activate Diversion Specialist");
        drain_stack(&mut g);
        assert!(g.battlefield_find(fodder).is_none(), "fodder sacrificed");
        assert!(g.exile.iter().any(|c| c.id == top), "top card exiled to play this turn");
    }

    /// Sumala Sentry counters itself and the turned card when a face-down
    /// permanent you control is turned face up.
    #[test]
    fn sumala_sentry_turn_face_up_counters() {
        let mut g = two_player_game();
        let ss = g.add_card_to_battlefield(0, catalog::sumala_sentry());
        let top = g.next_id();
        g.players[0].library.insert(0, crabomination::card::CardInstance::new(top, catalog::elder_gargaroth(), 0));
        let ctx = crabomination::game::effects::EffectContext::for_ability(top, 0, None);
        let mut events = vec![];
        g.manifest_card(top, 0, &ctx, &mut events);
        ready(&mut g);
        g.perform_action(GameAction::TurnFaceUp { card_id: top }).expect("turn face up");
        drain_stack(&mut g);
        assert_eq!(
            g.battlefield_find(ss).unwrap().counter_count(CounterType::PlusOnePlusOne),
            1,
            "Sumala got a +1/+1 counter",
        );
        assert_eq!(
            g.battlefield_find(top).unwrap().counter_count(CounterType::PlusOnePlusOne),
            1,
            "turned card got a +1/+1 counter",
        );
    }

    /// Cryptid Inspector counters itself when a face-down permanent enters and
    /// when one is turned face up.
    #[test]
    fn cryptid_inspector_face_down_matters() {
        let mut g = two_player_game();
        let ci = g.add_card_to_battlefield(0, catalog::cryptid_inspector());
        let top = g.next_id();
        g.players[0].library.insert(0, crabomination::card::CardInstance::new(top, catalog::elder_gargaroth(), 0));
        let ctx = crabomination::game::effects::EffectContext::for_ability(top, 0, None);
        let mut events = vec![];
        g.manifest_card(top, 0, &ctx, &mut events);
        g.dispatch_triggers_for_events(&events); // face-down permanent entered
        drain_stack(&mut g);
        assert_eq!(
            g.battlefield_find(ci).unwrap().counter_count(CounterType::PlusOnePlusOne),
            1,
            "+1/+1 when a face-down permanent entered",
        );
        ready(&mut g);
        g.perform_action(GameAction::TurnFaceUp { card_id: top }).expect("turn face up");
        drain_stack(&mut g);
        assert_eq!(
            g.battlefield_find(ci).unwrap().counter_count(CounterType::PlusOnePlusOne),
            2,
            "+1/+1 again when turned face up",
        );
    }

    /// Fanatic of the Harrowing makes each player discard, then its controller draws.
    #[test]
    fn fanatic_of_the_harrowing_discards_and_draws() {
        let mut g = two_player_game();
        g.add_card_to_hand(0, catalog::grizzly_bears()); // discard fodder
        g.add_card_to_hand(1, catalog::grizzly_bears());
        g.add_card_to_library(0, catalog::forest());
        let foh = g.add_card_to_hand(0, catalog::fanatic_of_the_harrowing());
        ready(&mut g);
        g.perform_action(GameAction::CastSpell {
            card_id: foh, target: None, additional_targets: vec![], mode: None, x_value: None,
        })
        .expect("cast Fanatic");
        drain_stack(&mut g);
        assert_eq!(g.players[1].graveyard.len(), 1, "opponent discarded one");
        // P0 discarded the fodder and drew the Forest: net hand back to one card.
        assert!(g.players[0].graveyard.iter().any(|c| c.definition.name == "Grizzly Bears"), "you discarded");
        assert!(g.players[0].hand.iter().any(|c| c.definition.name == "Forest"), "you drew");
    }

    /// Spectral Snatcher carries Ward—Discard and Swampcycling.
    #[test]
    fn spectral_snatcher_keywords() {
        let def = catalog::spectral_snatcher();
        assert!(def.keywords.iter().any(|k| matches!(k, Keyword::Ward(crabomination::card::WardCost::Discard(1)))));
        assert!(def.keywords.iter().any(|k| matches!(k, Keyword::Landcycling(_, crabomination::card::LandType::Swamp))));
    }

    /// Ghostly Keybearer carries a combat-damage trigger; the UnlockRoomDoor
    /// effect it fires opens a still-locked door of the targeted Room.
    #[test]
    fn ghostly_keybearer_unlocks_a_door() {
        use crabomination::card::Effect;
        use crabomination::game::effects::EffectContext;
        let mut g = two_player_game();
        let gk = g.add_card_to_battlefield(0, catalog::ghostly_keybearer());
        let def = catalog::ghostly_keybearer();
        assert_eq!(
            def.triggered_abilities[0].event.kind,
            crabomination::card::EventKind::DealsCombatDamageToPlayer,
            "fires on dealing combat damage to a player",
        );
        // A Room with its left door unlocked; the right is still locked.
        let room = g.add_card_to_hand(0, catalog::unholy_annex_ritual_chamber());
        ready(&mut g);
        g.perform_action(GameAction::CastRoomDoor { card_id: room, right: false })
            .expect("cast left door");
        drain_stack(&mut g);
        assert_eq!(g.battlefield_find(room).unwrap().unlocked_doors, 0b01, "only left open");
        // Resolve the unlock effect directly (the trigger body) against the Room.
        let ctx = EffectContext::for_ability(gk, 0, Some(Target::Permanent(room)));
        let evs = g
            .resolve_effect(
                &Effect::UnlockRoomDoor { what: crabomination::card::Selector::Target(0) },
                &ctx,
            )
            .expect("unlock effect");
        g.dispatch_triggers_for_events(&evs);
        drain_stack(&mut g);
        assert_eq!(g.battlefield_find(room).unwrap().unlocked_doors, 0b11, "right door unlocked");
    }

    /// Enduring Tenacity drains an opponent when you gain life.
    #[test]
    fn enduring_tenacity_drains_on_lifegain() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::enduring_tenacity());
        let foe = g.players[1].life;
        let ctx = crabomination::game::effects::EffectContext::for_ability(crabomination::card::CardId(999), 0, None);
        let events = g
            .resolve_effect(
                &crabomination::card::Effect::GainLife {
                    who: crabomination::card::Selector::You,
                    amount: crabomination::card::Value::Const(3),
                },
                &ctx,
            )
            .unwrap();
        g.dispatch_triggers_for_events(&events);
        drain_stack(&mut g);
        assert_eq!(g.players[1].life, foe - 3, "opponent lost the life gained");
    }

    /// Threats Around Every Corner manifests dread on ETB.
    #[test]
    fn threats_around_every_corner_manifests() {
        let mut g = two_player_game();
        g.add_card_to_library(0, catalog::grizzly_bears());
        g.add_card_to_library(0, catalog::forest());
        let tac = g.add_card_to_hand(0, catalog::threats_around_every_corner());
        ready(&mut g);
        g.perform_action(GameAction::CastSpell {
            card_id: tac, target: None, additional_targets: vec![], mode: None, x_value: None,
        })
        .expect("cast Threats");
        drain_stack(&mut g);
        assert!(g.battlefield.iter().any(|c| c.face_down), "manifested a face-down 2/2");
    }

    /// Insidious Fungus sacrifices itself to destroy an artifact.
    #[test]
    fn insidious_fungus_sacs_to_destroy_artifact() {
        let mut g = two_player_game();
        let fungus = g.add_card_to_battlefield(0, catalog::insidious_fungus());
        let art = g.add_card_to_battlefield(1, catalog::ornithopter());
        g.clear_sickness(fungus);
        ready(&mut g);
        g.perform_action(GameAction::ActivateAbility {
            card_id: fungus, ability_index: 0, target: Some(Target::Permanent(art)),
            additional_targets: vec![], x_value: None,
        })
        .expect("activate Insidious Fungus (mode 0)");
        drain_stack(&mut g);
        assert!(g.battlefield_find(fungus).is_none(), "Fungus sacrificed");
        assert!(g.battlefield_find(art).is_none(), "artifact destroyed");
    }

    /// Winter's Intervention deals 2 to a creature and gains 2 life.
    #[test]
    fn winters_intervention_burns_and_gains() {
        let mut g = two_player_game();
        let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
        let wi = g.add_card_to_hand(0, catalog::winters_intervention());
        ready(&mut g);
        let life = g.players[0].life;
        cast_at(&mut g, wi, Target::Permanent(foe));
        assert!(g.battlefield_find(foe).is_none(), "2 kills the 2/2");
        assert_eq!(g.players[0].life, life + 2, "gained 2 life");
    }

    /// Shroudstomper drains and draws on enter.
    #[test]
    fn shroudstomper_etb_drains_and_draws() {
        let mut g = two_player_game();
        g.add_card_to_library(0, catalog::forest());
        let ss = g.add_card_to_hand(0, catalog::shroudstomper());
        ready(&mut g);
        let (foe, life, hand) = (g.players[1].life, g.players[0].life, g.players[0].hand.len());
        g.perform_action(GameAction::CastSpell {
            card_id: ss, target: None, additional_targets: vec![], mode: None, x_value: None,
        })
        .expect("cast Shroudstomper");
        drain_stack(&mut g);
        assert_eq!(g.players[1].life, foe - 2, "opponent lost 2");
        assert_eq!(g.players[0].life, life + 2, "gained 2");
        // Hand: -1 (cast Shroudstomper) +1 (drew) = net same.
        assert_eq!(g.players[0].hand.len(), hand, "drew a card");
    }

    /// Patched Plaything enters with two -1/-1 counters only when cast from hand.
    #[test]
    fn patched_plaything_cast_zone_counters() {
        let mut g = two_player_game();
        // Cast from hand → enters as a 2/1 with two -1/-1 counters.
        let pp = g.add_card_to_hand(0, catalog::patched_plaything());
        ready(&mut g);
        g.perform_action(GameAction::CastSpell {
            card_id: pp, target: None, additional_targets: vec![], mode: None, x_value: None,
        })
        .expect("cast Patched Plaything");
        drain_stack(&mut g);
        assert_eq!(
            g.battlefield_find(pp).unwrap().counter_count(CounterType::MinusOneMinusOne),
            2,
            "hand-cast enters with two -1/-1 counters",
        );

        // Entering any other way (here: straight onto the battlefield) skips them.
        let direct = g.add_card_to_battlefield(0, catalog::patched_plaything());
        assert_eq!(
            g.battlefield_find(direct).unwrap().counter_count(CounterType::MinusOneMinusOne),
            0,
            "non-hand entry has no -1/-1 counters",
        );
    }

    /// Broadside Barrage deals 5 and loots.
    #[test]
    fn broadside_barrage_burns_and_loots() {
        let mut g = two_player_game();
        let foe = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4
        g.add_card_to_library(0, catalog::grizzly_bears());
        let bb = g.add_card_to_hand(0, catalog::broadside_barrage());
        ready(&mut g);
        cast_at(&mut g, bb, Target::Permanent(foe));
        assert!(g.battlefield_find(foe).is_none(), "5 kills the 4/4");
    }

    /// Spin Out destroys a creature.
    #[test]
    fn spin_out_destroys_creature() {
        let mut g = two_player_game();
        let foe = g.add_card_to_battlefield(1, catalog::serra_angel());
        let so = g.add_card_to_hand(0, catalog::spin_out());
        ready(&mut g);
        cast_at(&mut g, so, Target::Permanent(foe));
        assert!(g.battlefield_find(foe).is_none());
    }

    /// Syphon Fuel shrinks a creature and gains life.
    #[test]
    fn syphon_fuel_shrinks_and_gains() {
        let mut g = two_player_game();
        let foe = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4
        let sf = g.add_card_to_hand(0, catalog::syphon_fuel());
        ready(&mut g);
        let life = g.players[0].life;
        cast_at(&mut g, sf, Target::Permanent(foe));
        assert!(g.battlefield_find(foe).is_none(), "-6/-6 kills the 4/4");
        assert_eq!(g.players[0].life, life + 2, "gained 2 life");
    }

    /// Locust Spray gives -1/-1; it can also cycle.
    #[test]
    fn locust_spray_weakens() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
        let ls = g.add_card_to_hand(0, catalog::locust_spray());
        ready(&mut g);
        cast_at(&mut g, ls, Target::Permanent(bear));
        let cp = g.computed_permanent(bear).unwrap();
        assert_eq!((cp.power, cp.toughness), (1, 1));
    }

    /// Skycrash destroys an artifact.
    #[test]
    fn skycrash_destroys_artifact() {
        let mut g = two_player_game();
        let art = g.add_card_to_battlefield(1, catalog::air_response_unit());
        let sc = g.add_card_to_hand(0, catalog::skycrash());
        ready(&mut g);
        cast_at(&mut g, sc, Target::Permanent(art));
        assert!(g.battlefield_find(art).is_none());
    }

    /// Maximum Overdrive adds a counter and grants indestructible + deathtouch.
    #[test]
    fn maximum_overdrive_buffs() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let mo = g.add_card_to_hand(0, catalog::maximum_overdrive());
        ready(&mut g);
        cast_at(&mut g, mo, Target::Permanent(bear));
        let cp = g.computed_permanent(bear).unwrap();
        assert_eq!((cp.power, cp.toughness), (3, 3), "+1/+1 counter");
        assert!(cp.keywords.contains(&Keyword::Indestructible));
        assert!(cp.keywords.contains(&Keyword::Deathtouch));
    }

    /// Pedal to the Metal pumps +X/+0 where X is the cast X.
    #[test]
    fn pedal_to_the_metal_pumps_by_x() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
        let p = g.add_card_to_hand(0, catalog::pedal_to_the_metal());
        ready(&mut g);
        g.perform_action(GameAction::CastSpell {
            card_id: p,
            target: Some(Target::Permanent(bear)),
            additional_targets: vec![],
            mode: None,
            x_value: Some(3),
        })
        .expect("cast Pedal with X=3");
        drain_stack(&mut g);
        let cp = g.computed_permanent(bear).unwrap();
        assert_eq!(cp.power, 5, "+3/+0");
        assert!(cp.keywords.contains(&Keyword::FirstStrike));
    }

    /// Fuel the Flames deals 2 to each creature.
    #[test]
    fn fuel_the_flames_sweeps_for_two() {
        let mut g = two_player_game();
        let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
        let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
        let big = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4
        let ff = g.add_card_to_hand(0, catalog::fuel_the_flames());
        ready(&mut g);
        g.perform_action(GameAction::CastSpell {
            card_id: ff, target: None, additional_targets: vec![], mode: None, x_value: None,
        })
        .expect("cast");
        drain_stack(&mut g);
        assert!(g.battlefield_find(mine).is_none(), "2 dmg kills 2/2");
        assert!(g.battlefield_find(foe).is_none());
        assert!(g.battlefield_find(big).is_some(), "4/4 survives");
    }

    /// Gallant Strike destroys only a toughness-4+ creature.
    #[test]
    fn gallant_strike_hits_big_toughness() {
        let mut g = two_player_game();
        let big = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4
        let gs = g.add_card_to_hand(0, catalog::gallant_strike());
        ready(&mut g);
        cast_at(&mut g, gs, Target::Permanent(big));
        assert!(g.battlefield_find(big).is_none());
    }

    /// Risky Shortcut draws two and drains each player 2.
    #[test]
    fn risky_shortcut_draws_and_drains() {
        let mut g = two_player_game();
        g.add_card_to_library(0, catalog::grizzly_bears());
        g.add_card_to_library(0, catalog::grizzly_bears());
        let rs = g.add_card_to_hand(0, catalog::risky_shortcut());
        ready(&mut g);
        let (l0, l1) = (g.players[0].life, g.players[1].life);
        g.perform_action(GameAction::CastSpell {
            card_id: rs, target: None, additional_targets: vec![], mode: None, x_value: None,
        })
        .expect("cast");
        drain_stack(&mut g);
        assert_eq!(g.players[0].hand.len(), 2, "drew two");
        assert_eq!(g.players[0].life, l0 - 2);
        assert_eq!(g.players[1].life, l1 - 2);
    }

    /// Road Rage's X scales with Mounts and Vehicles you control (2 + count).
    #[test]
    fn road_rage_scales_with_vehicles() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::air_response_unit()); // a Vehicle
        g.add_card_to_battlefield(0, catalog::debris_beetle()); // another Vehicle
        let foe = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4
        let rr = g.add_card_to_hand(0, catalog::road_rage());
        ready(&mut g);
        cast_at(&mut g, rr, Target::Permanent(foe));
        // X = 2 + 2 vehicles = 4 → kills the 4/4.
        assert!(g.battlefield_find(foe).is_none(), "4 damage kills the 4/4");
    }

    /// Spectacular Pileup destroys all creatures and Vehicles, even indestructible.
    #[test]
    fn spectacular_pileup_wraths_everything() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let veh = g.add_card_to_battlefield(1, catalog::air_response_unit());
        let sp = g.add_card_to_hand(0, catalog::spectacular_pileup());
        ready(&mut g);
        g.perform_action(GameAction::CastSpell {
            card_id: sp, target: None, additional_targets: vec![], mode: None, x_value: None,
        })
        .expect("cast");
        drain_stack(&mut g);
        assert!(g.battlefield_find(bear).is_none(), "creature destroyed");
        assert!(g.battlefield_find(veh).is_none(), "Vehicle destroyed");
    }

    /// Nimble Thopterist mints a 1/1 flying Thopter on ETB.
    #[test]
    fn nimble_thopterist_makes_thopter() {
        let mut g = two_player_game();
        let nt = g.add_card_to_battlefield(0, catalog::nimble_thopterist());
        g.fire_self_etb_triggers(nt, 0);
        drain_stack(&mut g);
        assert_eq!(count_named(&g, 0, "Thopter"), 1);
    }

    /// Shefet Archfiend gives all other creatures -2/-2 on ETB.
    #[test]
    fn shefet_archfiend_sweeps_others() {
        let mut g = two_player_game();
        let x = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
        let sa = g.add_card_to_battlefield(0, catalog::shefet_archfiend());
        g.fire_self_etb_triggers(sa, 0);
        drain_stack(&mut g);
        assert!(g.battlefield_find(x).is_none(), "-2/-2 kills the 2/2");
        assert!(g.battlefield_find(sa).is_some(), "Archfiend itself unaffected");
    }

    /// Regal Imperiosaur is a Dinosaur lord (other Dinosaurs +1/+1).
    #[test]
    fn regal_imperiosaur_buffs_dinosaurs() {
        let mut g = two_player_game();
        let other = g.add_card_to_battlefield(0, catalog::migrating_ketradon()); // 6/6 Dino
        g.add_card_to_battlefield(0, catalog::regal_imperiosaur());
        let cp = g.computed_permanent(other).unwrap();
        assert_eq!((cp.power, cp.toughness), (7, 7), "lord gives +1/+1");
    }

    /// Guidelight Synergist grows with artifacts you control.
    #[test]
    fn guidelight_synergist_scales_with_artifacts() {
        let mut g = two_player_game();
        let gs = g.add_card_to_battlefield(0, catalog::guidelight_synergist()); // 0/4, an artifact
        // Counts itself.
        assert_eq!(g.computed_permanent(gs).unwrap().power, 1, "+1/+0 for itself");
        g.add_card_to_battlefield(0, catalog::air_response_unit()); // +1 artifact
        assert_eq!(g.computed_permanent(gs).unwrap().power, 2);
    }

    /// Cloudspire Captain buffs Mounts and Vehicles you control.
    #[test]
    fn cloudspire_captain_buffs_vehicles() {
        let mut g = two_player_game();
        let veh = g.add_card_to_battlefield(0, catalog::air_response_unit()); // 3/3 Vehicle
        g.add_card_to_battlefield(0, catalog::cloudspire_captain());
        let cp = g.computed_permanent(veh).unwrap();
        assert_eq!((cp.power, cp.toughness), (4, 4), "+1/+1 anthem");
    }

    /// Daring Mechanic puts a +1/+1 counter on a Vehicle.
    #[test]
    fn daring_mechanic_counters_vehicle() {
        let mut g = two_player_game();
        let veh = g.add_card_to_battlefield(0, catalog::debris_beetle()); // 6/6
        let dm = g.add_card_to_battlefield(0, catalog::daring_mechanic());
        g.clear_sickness(dm);
        ready(&mut g);
        g.perform_action(GameAction::ActivateAbility {
            card_id: dm, ability_index: 0, target: Some(Target::Permanent(veh)),
            additional_targets: vec![], x_value: None,
        })
        .expect("activate");
        drain_stack(&mut g);
        let cp = g.computed_permanent(veh).unwrap();
        assert_eq!((cp.power, cp.toughness), (7, 7), "+1/+1 counter");
    }

    /// Deathless Pilot returns itself from the graveyard.
    #[test]
    fn deathless_pilot_recurs_from_graveyard() {
        let mut g = two_player_game();
        let dp = g.add_card_to_graveyard(0, catalog::deathless_pilot());
        ready(&mut g);
        g.perform_action(GameAction::ActivateAbility {
            card_id: dp, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
        })
        .expect("activate gy ability");
        drain_stack(&mut g);
        assert_eq!(g.players[0].hand.iter().filter(|c| c.definition.name == "Deathless Pilot").count(), 1);
    }

    /// Debris Beetle drains 3 on enter (Vehicle ETB).
    #[test]
    fn debris_beetle_drains_on_etb() {
        let mut g = two_player_game();
        let (l0, l1) = (g.players[0].life, g.players[1].life);
        let db = g.add_card_to_battlefield(0, catalog::debris_beetle());
        g.fire_self_etb_triggers(db, 0);
        drain_stack(&mut g);
        assert_eq!(g.players[1].life, l1 - 3);
        assert_eq!(g.players[0].life, l0 + 3);
    }

    /// Cryptcaller Chariot mints a tapped Zombie per discarded card.
    #[test]
    fn cryptcaller_chariot_makes_zombies_on_discard() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::cryptcaller_chariot());
        let card = g.add_card_to_hand(0, catalog::grizzly_bears());
        let mut events = vec![];
        g.discard_card(0, card, &mut events);
        g.dispatch_triggers_for_events(&events);
        drain_stack(&mut g);
        assert_eq!(count_named(&g, 0, "Zombie"), 1, "one Zombie per discard");
    }

    /// Scrounging Skyray grows when you discard.
    #[test]
    fn scrounging_skyray_grows_on_discard() {
        let mut g = two_player_game();
        let sky = g.add_card_to_battlefield(0, catalog::scrounging_skyray()); // 1/2
        let card = g.add_card_to_hand(0, catalog::grizzly_bears());
        let mut events = vec![];
        g.discard_card(0, card, &mut events);
        g.dispatch_triggers_for_events(&events);
        drain_stack(&mut g);
        assert_eq!(
            g.battlefield_find(sky).unwrap().counter_count(CounterType::PlusOnePlusOne),
            1,
        );
    }

    /// Pactdoll Terror drains 1 when an artifact you control enters.
    #[test]
    fn pactdoll_terror_drains_on_artifact_etb() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::pactdoll_terror());
        let (l0, l1) = (g.players[0].life, g.players[1].life);
        let veh = g.add_card_to_battlefield(0, catalog::air_response_unit());
        g.dispatch_triggers_for_events(&[GameEvent::PermanentEntered { card_id: veh }]);
        drain_stack(&mut g);
        assert_eq!(g.players[1].life, l1 - 1);
        assert_eq!(g.players[0].life, l0 + 1);
    }

    /// Cloudspire Skycycle distributes two +1/+1 counters on ETB.
    #[test]
    fn cloudspire_skycycle_distributes_counters() {
        let mut g = two_player_game();
        let target = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
        let sky = g.add_card_to_battlefield(0, catalog::cloudspire_skycycle());
        g.fire_self_etb_triggers(sky, 0);
        drain_stack(&mut g);
        // Two counters land on the single eligible other creature.
        assert_eq!(
            g.battlefield_find(target).unwrap().counter_count(CounterType::PlusOnePlusOne),
            2,
        );
    }

    /// Deathless Pilot's CR 702.122e rider lets a 2-power creature crew a Crew 4
    /// Vehicle by itself (counts as power 4).
    #[test]
    fn deathless_pilot_crews_as_though_power_greater() {
        let mut g = two_player_game();
        let veh = g.add_card_to_battlefield(0, catalog::debris_beetle()); // Crew 2... use a Crew 4
        // Debris Beetle is Crew 2; pair the pilot with a Crew 4 vehicle instead.
        g.battlefield.retain(|c| c.id != veh);
        let chariot = g.add_card_to_battlefield(0, catalog::lumbering_worldwagon()); // Crew 4
        let pilot = g.add_card_to_battlefield(0, catalog::deathless_pilot()); // power 2 (+2 rider = 4)
        g.clear_sickness(pilot);
        g.active_player_idx = 0;
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::Crew { vehicle: chariot, crew_creatures: vec![pilot] })
            .expect("2-power pilot crews Crew 4 via the +2 rider");
        assert!(
            g.computed_permanent(chariot).unwrap().card_types.contains(&crabomination::card::CardType::Creature),
            "Vehicle is crewed (an artifact creature)",
        );
    }

    /// Thunderhead Gunner loots: discard a card to draw one.
    #[test]
    fn thunderhead_gunner_loots() {
        let mut g = two_player_game();
        let tg = g.add_card_to_battlefield(0, catalog::thunderhead_gunner());
        g.clear_sickness(tg);
        g.add_card_to_hand(0, catalog::grizzly_bears()); // a card to discard
        g.add_card_to_library(0, catalog::forest()); // a card to draw
        g.active_player_idx = 0;
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        let hand_before = g.players[0].hand.len();
        g.perform_action(GameAction::ActivateAbility {
            card_id: tg, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
        })
        .expect("activate loot");
        drain_stack(&mut g);
        // -1 discard, +1 draw → net unchanged, but the drawn card differs.
        assert_eq!(g.players[0].hand.len(), hand_before, "discard 1, draw 1");
    }

    /// Wretched Doll surveils 1.
    #[test]
    fn wretched_doll_surveils() {
        let mut g = two_player_game();
        let wd = g.add_card_to_battlefield(0, catalog::wretched_doll());
        g.clear_sickness(wd);
        g.add_card_to_library(0, catalog::forest());
        g.players[0].mana_pool.add(Color::Black, 1);
        g.active_player_idx = 0;
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::ActivateAbility {
            card_id: wd, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
        })
        .expect("activate surveil");
        drain_stack(&mut g);
        assert!(g.battlefield_find(wd).is_some(), "Doll stays (surveil resolved)");
    }

    /// Molt Tender mills with its first ability.
    #[test]
    fn molt_tender_mills() {
        let mut g = two_player_game();
        let mt = g.add_card_to_battlefield(0, catalog::molt_tender());
        g.clear_sickness(mt);
        g.add_card_to_library(0, catalog::forest());
        g.active_player_idx = 0;
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        let gy_before = g.players[0].graveyard.len();
        g.perform_action(GameAction::ActivateAbility {
            card_id: mt, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
        })
        .expect("activate mill");
        drain_stack(&mut g);
        assert_eq!(g.players[0].graveyard.len(), gy_before + 1, "milled one card");
    }

    /// Scrap Compactor's first ability deals 3 to a creature (sacrificing itself).
    #[test]
    fn scrap_compactor_pings_for_three() {
        let mut g = two_player_game();
        let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
        let sc = g.add_card_to_battlefield(0, catalog::scrap_compactor());
        g.clear_sickness(sc);
        g.players[0].mana_pool.add_colorless(3);
        g.active_player_idx = 0;
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::ActivateAbility {
            card_id: sc, ability_index: 0, target: Some(Target::Permanent(foe)),
            additional_targets: vec![], x_value: None,
        })
        .expect("activate ping");
        drain_stack(&mut g);
        assert!(g.battlefield_find(foe).is_none(), "3 damage kills the 2/2");
        assert!(g.battlefield_find(sc).is_none(), "Compactor sacrificed itself");
    }

    /// Defend the Rider can make a 1/1 Pilot token.
    #[test]
    fn defend_the_rider_makes_pilot() {
        let mut g = two_player_game();
        let dr = g.add_card_to_hand(0, catalog::defend_the_rider());
        ready(&mut g);
        // Mode 2 (token) is chosen by the auto-decider when no controlled
        // permanent exists to target for mode 1.
        g.perform_action(GameAction::CastSpell {
            card_id: dr, target: None, additional_targets: vec![], mode: Some(1), x_value: None,
        })
        .expect("cast Defend the Rider (token mode)");
        drain_stack(&mut g);
        assert_eq!(count_named(&g, 0, "Pilot"), 1);
    }

    /// Full Throttle adds two combat phases after the main phase and untaps
    /// attackers at the beginning of each combat this turn.
    #[test]
    fn full_throttle_adds_two_combats() {
        let mut g = two_player_game();
        let ft = g.add_card_to_hand(0, catalog::full_throttle());
        let atk = g.add_card_to_battlefield(0, catalog::canyon_vaulter());
        ready(&mut g);
        let combats_before = g.additional_post_main_combats;
        g.perform_action(GameAction::CastSpell {
            card_id: ft, target: None, additional_targets: vec![], mode: None, x_value: None,
        })
        .expect("cast Full Throttle");
        drain_stack(&mut g);
        assert_eq!(g.additional_post_main_combats, combats_before + 2, "two additional combats queued");

        // Mark the creature as a tapped attacker, then enter a fresh Begin
        // Combat: the delayed rider untaps it so it can attack again.
        {
            let c = g.battlefield_find_mut(atk).unwrap();
            c.tapped = true;
            c.attacked_this_turn = true;
        }
        g.fire_step_triggers(TurnStep::BeginCombat);
        drain_stack(&mut g);
        assert!(!g.battlefield_find(atk).unwrap().tapped, "attacker untapped for next combat");
    }

    /// Canyon Vaulter's crew trigger (CR 702.122) gives the crewed Vehicle flying.
    #[test]
    fn canyon_vaulter_crew_grants_flying() {
        let mut g = two_player_game();
        let veh = g.add_card_to_battlefield(0, catalog::debris_beetle()); // Crew 2, no flying
        let cv = g.add_card_to_battlefield(0, catalog::canyon_vaulter()); // power 3
        g.clear_sickness(cv);
        g.active_player_idx = 0;
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::Crew { vehicle: veh, crew_creatures: vec![cv] })
            .expect("crew");
        drain_stack(&mut g);
        assert!(
            g.computed_permanent(veh).unwrap().keywords.contains(&Keyword::Flying),
            "crewed Vehicle gained flying from Canyon Vaulter's trigger",
        );
    }

    /// Reckless Velocitaur's crew trigger pumps the crewed Vehicle +2/+0 trample.
    #[test]
    fn reckless_velocitaur_crew_pumps() {
        let mut g = two_player_game();
        let veh = g.add_card_to_battlefield(0, catalog::air_response_unit()); // 3/3 Crew 1
        let rv = g.add_card_to_battlefield(0, catalog::reckless_velocitaur());
        g.clear_sickness(rv);
        g.active_player_idx = 0;
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::Crew { vehicle: veh, crew_creatures: vec![rv] })
            .expect("crew");
        drain_stack(&mut g);
        let cp = g.computed_permanent(veh).unwrap();
        assert_eq!(cp.power, 5, "+2/+0");
        assert!(cp.keywords.contains(&Keyword::Trample));
    }

    /// The crew trigger is gated to the controller's main phase: crewing during
    /// combat (instant speed) doesn't fire it.
    #[test]
    fn crew_trigger_silent_outside_main_phase() {
        let mut g = two_player_game();
        let veh = g.add_card_to_battlefield(0, catalog::debris_beetle()); // no flying
        let cv = g.add_card_to_battlefield(0, catalog::canyon_vaulter());
        g.clear_sickness(cv);
        g.active_player_idx = 0;
        g.step = TurnStep::BeginCombat; // not a main phase
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::Crew { vehicle: veh, crew_creatures: vec![cv] })
            .expect("crew");
        drain_stack(&mut g);
        assert!(
            !g.computed_permanent(veh).unwrap().keywords.contains(&Keyword::Flying),
            "no flying — crew happened outside the main phase",
        );
    }

    /// Emerge from the Cocoon reanimates a creature from the graveyard.
    #[test]
    fn emerge_from_the_cocoon_reanimates() {
        let mut g = two_player_game();
        let dead = g.add_card_to_graveyard(0, catalog::grizzly_bears());
        let e = g.add_card_to_hand(0, catalog::emerge_from_the_cocoon());
        ready(&mut g);
        let life = g.players[0].life;
        cast_at(&mut g, e, Target::Permanent(dead));
        assert!(g.battlefield_find(dead).is_some(), "Bears reanimated to the battlefield");
        assert_eq!(g.players[0].life, life + 3, "gained 3");
    }

    /// Enter the Enigma makes a creature unblockable and draws.
    #[test]
    fn enter_the_enigma_unblockable_and_draws() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.add_card_to_library(0, catalog::forest());
        let e = g.add_card_to_hand(0, catalog::enter_the_enigma());
        ready(&mut g);
        let hand = g.players[0].hand.len();
        cast_at(&mut g, e, Target::Permanent(bear));
        assert!(g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Unblockable));
        assert_eq!(g.players[0].hand.len(), hand, "spell left hand (-1) and drew (+1)");
    }

    /// Exorcise exiles a power-4+ creature.
    #[test]
    fn exorcise_exiles_big_creature() {
        let mut g = two_player_game();
        let big = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4
        let ex = g.add_card_to_hand(0, catalog::exorcise());
        ready(&mut g);
        cast_at(&mut g, ex, Target::Permanent(big));
        assert!(g.battlefield_find(big).is_none(), "4/4 exiled");
    }

    /// Fear of Lost Teeth pings on death and gains a life.
    #[test]
    fn fear_of_lost_teeth_dies_pings() {
        let mut g = two_player_game();
        let f = g.add_card_to_battlefield(0, catalog::fear_of_lost_teeth());
        g.battlefield_find_mut(f).unwrap().damage = 1; // lethal vs 1 toughness
        let life = g.players[0].life;
        let foe_life = g.players[1].life;
        g.priority.player_with_priority = 0;
        g.check_state_based_actions();
        drain_stack(&mut g);
        assert_eq!(g.players[0].life, life + 1, "gained 1");
        assert_eq!(g.players[1].life, foe_life - 1, "pinged the opponent");
    }

    /// Friendly Teddy draws for each player on death.
    #[test]
    fn friendly_teddy_dies_each_draws() {
        let mut g = two_player_game();
        g.add_card_to_library(0, catalog::forest());
        g.add_card_to_library(1, catalog::forest());
        let t = g.add_card_to_battlefield(0, catalog::friendly_teddy());
        g.battlefield_find_mut(t).unwrap().damage = 2;
        let (h0, h1) = (g.players[0].hand.len(), g.players[1].hand.len());
        g.check_state_based_actions();
        drain_stack(&mut g);
        assert_eq!(g.players[0].hand.len(), h0 + 1);
        assert_eq!(g.players[1].hand.len(), h1 + 1);
    }

    /// Give In to Violence pumps +2/+2 and grants lifelink.
    #[test]
    fn give_in_to_violence_pumps_lifelink() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
        let giv = g.add_card_to_hand(0, catalog::give_in_to_violence());
        ready(&mut g);
        cast_at(&mut g, giv, Target::Permanent(bear));
        let cp = g.computed_permanent(bear).unwrap();
        assert_eq!((cp.power, cp.toughness), (4, 4));
        assert!(cp.keywords.contains(&Keyword::Lifelink));
    }

    /// Grasping Longneck gains 2 life when it dies; it has reach.
    #[test]
    fn grasping_longneck_reach_and_dies_gain() {
        let mut g = two_player_game();
        let gl = g.add_card_to_battlefield(0, catalog::grasping_longneck());
        assert!(g.computed_permanent(gl).unwrap().keywords.contains(&Keyword::Reach));
        g.battlefield_find_mut(gl).unwrap().damage = 2; // lethal vs 2 toughness
        let life = g.players[0].life;
        g.check_state_based_actions();
        drain_stack(&mut g);
        assert_eq!(g.players[0].life, life + 2);
    }

    /// Horrid Vigor grants deathtouch and indestructible.
    #[test]
    fn horrid_vigor_grants_deathtouch_indestructible() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let hv = g.add_card_to_hand(0, catalog::horrid_vigor());
        ready(&mut g);
        cast_at(&mut g, hv, Target::Permanent(bear));
        let cp = g.computed_permanent(bear).unwrap();
        assert!(cp.keywords.contains(&Keyword::Deathtouch));
        assert!(cp.keywords.contains(&Keyword::Indestructible));
    }

    /// Glimmerburst draws two and makes a Glimmer token.
    #[test]
    fn glimmerburst_draws_and_makes_glimmer() {
        let mut g = two_player_game();
        g.add_card_to_library(0, catalog::forest());
        g.add_card_to_library(0, catalog::forest());
        let gb = g.add_card_to_hand(0, catalog::glimmerburst());
        ready(&mut g);
        let hand = g.players[0].hand.len();
        g.perform_action(GameAction::CastSpell {
            card_id: gb, target: None, additional_targets: vec![], mode: None, x_value: None,
        })
        .expect("cast");
        drain_stack(&mut g);
        assert_eq!(g.players[0].hand.len(), hand - 1 + 2, "-1 cast, +2 drawn");
        assert_eq!(count_named(&g, 0, "Glimmer"), 1);
    }

    /// Friendly Ghost flies and pumps a creature +2/+4 on ETB.
    #[test]
    fn friendly_ghost_etb_pumps() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
        let fg = g.add_card_to_battlefield(0, catalog::friendly_ghost());
        assert!(g.computed_permanent(fg).unwrap().keywords.contains(&Keyword::Flying));
        g.fire_self_etb_triggers(fg, 0);
        drain_stack(&mut g);
        let cp = g.computed_permanent(bear).unwrap();
        assert_eq!((cp.power, cp.toughness), (4, 6), "+2/+4");
    }

    /// Air Response Unit ships as a 3/3 Vehicle with Crew 1.
    #[test]
    fn air_response_unit_is_crewable_vehicle() {
        let mut g = two_player_game();
        let v = g.add_card_to_battlefield(0, catalog::air_response_unit());
        let c = g.battlefield_find(v).unwrap();
        assert!(c.definition.keywords.contains(&Keyword::Crew(1)));
        assert_eq!((c.definition.power, c.definition.toughness), (3, 3));
    }

    /// Sawblade Skinripper grows on its sac ability and, at end step, deals damage
    /// equal to the number of permanents sacrificed this turn to any target.
    #[test]
    fn sawblade_skinripper_sac_payoff() {
        let mut g = two_player_game();
        let saw = g.add_card_to_battlefield(0, catalog::sawblade_skinripper());
        let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.clear_sickness(saw);
        ready(&mut g);
        g.perform_action(GameAction::ActivateAbility {
            card_id: saw, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
        })
        .expect("activate Sawblade sac ability");
        drain_stack(&mut g);
        assert!(g.battlefield_find(fodder).is_none(), "fodder sacrificed");
        assert_eq!(
            g.battlefield_find(saw).unwrap().counter_count(CounterType::PlusOnePlusOne),
            1,
            "Sawblade got a +1/+1 counter",
        );
        assert_eq!(g.players[0].permanents_sacrificed_this_turn, 1);
        // End step: 1 permanent sacrificed → 1 damage to the opponent.
        advance_to(&mut g, TurnStep::End);
        drain_stack(&mut g);
        assert_eq!(g.players[1].life, 19, "1 damage from the end-step trigger");
    }

    /// A minimal 1/1 white creature token (test fixture).
    fn soldier_1_1() -> TokenDefinition {
        TokenDefinition {
            name: "Soldier".into(),
            power: 1,
            toughness: 1,
            card_types: vec![CardType::Creature],
            colors: vec![Color::White],
            subtypes: Subtypes { creature_types: vec![CreatureType::Soldier], ..Default::default() },
            ..Default::default()
        }
    }

    /// Toby enters and mints a 4/4 Beast that can't attack or block alone.
    #[test]
    fn toby_makes_lonely_beast() {
        let mut g = two_player_game();
        let toby = g.add_card_to_battlefield(0, catalog::toby_beastie_befriender());
        g.fire_self_etb_triggers(toby, 0);
        drain_stack(&mut g);
        let beast = g
            .battlefield
            .iter()
            .find(|c| c.controller == 0 && c.definition.name == "Beast")
            .expect("Beast token created");
        assert_eq!((beast.definition.power, beast.definition.toughness), (4, 4));
        assert!(beast.definition.keywords.contains(&Keyword::CantAttackOrBlockAlone));
    }

    /// A creature with CantAttackOrBlockAlone can't be the only attacker.
    #[test]
    fn cant_attack_or_block_alone_blocks_lone_attack() {
        let mut g = two_player_game();
        let beast = g.add_token_to_battlefield(0, &soldier_with_alone());
        g.clear_sickness(beast);
        advance_to(&mut g, TurnStep::DeclareAttackers);
        let res = g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker: beast,
            target: AttackTarget::Player(1),
        }]));
        assert!(res.is_err(), "lone attack with can't-attack-alone is illegal");
    }

    /// Toby's anthem grants flying to your creature tokens once you control four.
    #[test]
    fn toby_anthem_grants_flying_at_four_tokens() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::toby_beastie_befriender());
        let t1 = g.add_token_to_battlefield(0, &soldier_1_1());
        // Three tokens: below the threshold, no flying yet.
        g.add_token_to_battlefield(0, &soldier_1_1());
        g.add_token_to_battlefield(0, &soldier_1_1());
        assert!(!g.computed_permanent(t1).unwrap().keywords.contains(&Keyword::Flying));
        // Fourth token trips the anthem.
        g.add_token_to_battlefield(0, &soldier_1_1());
        assert!(g.computed_permanent(t1).unwrap().keywords.contains(&Keyword::Flying));
    }

    /// A token with CantAttackOrBlockAlone — a soldier fixture for the combat test.
    fn soldier_with_alone() -> TokenDefinition {
        let mut t = soldier_1_1();
        t.keywords = vec![Keyword::CantAttackOrBlockAlone];
        t
    }

    /// A creature with CantAttackOrBlockAlone can't be the only blocker (CR 509.1c).
    #[test]
    fn cant_block_alone_rejects_lone_block() {
        let mut g = two_player_game();
        let atk = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let lone = g.add_token_to_battlefield(1, &soldier_with_alone());
        g.clear_sickness(atk);
        g.clear_sickness(lone);
        advance_to(&mut g, TurnStep::DeclareAttackers);
        g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker: atk, target: AttackTarget::Player(1),
        }])).expect("attack");
        drain_stack(&mut g);
        advance_to(&mut g, TurnStep::DeclareBlockers);
        let err = g.perform_action(GameAction::DeclareBlockers(vec![(lone, atk)]));
        assert!(err.is_err(), "can't block alone");
    }

    /// Twitching Doll's mana ability adds a nest counter; sacrificing it makes a
    /// Spider per counter (CR 605.1a incidental rider + LKI counter read).
    #[test]
    fn twitching_doll_nests_then_sacs_for_spiders() {
        let mut g = two_player_game();
        let doll = g.add_card_to_battlefield(0, catalog::twitching_doll());
        g.clear_sickness(doll);
        ready(&mut g);
        // Mana ability twice → two nest counters; pool gains two mana total.
        for _ in 0..2 {
            g.battlefield_find_mut(doll).unwrap().tapped = false;
            g.perform_action(GameAction::ActivateAbility {
                card_id: doll, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
            })
            .expect("mana + nest counter");
        }
        assert_eq!(g.battlefield_find(doll).unwrap().counter_count(CounterType::Nest), 2);
        // Sacrifice ability: make a Spider per counter (2).
        g.battlefield_find_mut(doll).unwrap().tapped = false;
        g.perform_action(GameAction::ActivateAbility {
            card_id: doll, ability_index: 1, target: None, additional_targets: vec![], x_value: None,
        })
        .expect("sacrifice for Spiders");
        drain_stack(&mut g);
        assert!(g.battlefield_find(doll).is_none(), "Doll sacrificed");
        assert_eq!(count_named(&g, 0, "Spider"), 2, "two Spiders from two counters");
    }

    /// Fear of Isolation costs an extra "return a permanent you control"; cast it
    /// and the bounce happens while it enters.
    #[test]
    fn fear_of_isolation_bounces_a_permanent() {
        let mut g = two_player_game();
        let land = g.add_card_to_battlefield(0, catalog::island());
        let foi = g.add_card_to_hand(0, catalog::fear_of_isolation());
        ready(&mut g);
        g.perform_action(GameAction::CastSpell {
            card_id: foi, target: None, additional_targets: vec![], mode: None, x_value: None,
        })
        .expect("cast Fear of Isolation");
        drain_stack(&mut g);
        assert!(g.players[0].hand.iter().any(|c| c.id == land), "permanent returned to hand");
        assert_eq!(count_named(&g, 0, "Fear of Isolation"), 1, "enchantment creature entered");
    }

    /// Trapped in the Screen exiles an opponent's permanent on ETB and gives it
    /// back when the enchantment leaves (linked exile, CR 603.6e).
    #[test]
    fn trapped_in_the_screen_exiles_until_it_leaves() {
        let mut g = two_player_game();
        let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let trap = g.add_card_to_battlefield(0, catalog::trapped_in_the_screen());
        g.fire_self_etb_triggers(trap, 0);
        drain_stack(&mut g);
        assert!(g.battlefield_find(foe).is_none(), "opponent's creature exiled");
        // Destroy the enchantment → linked exile returns the creature.
        g.remove_to_graveyard_with_triggers(trap);
        drain_stack(&mut g);
        assert!(g.battlefield.iter().any(|c| c.id == foe), "creature returns when Trapped leaves");
    }

    /// Sheltered by Ghosts enchants your creature (+1/+0, lifelink, ward) and
    /// exiles an opponent's nonland permanent on ETB.
    #[test]
    fn sheltered_by_ghosts_buffs_and_exiles() {
        let mut g = two_player_game();
        let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
        let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let aura = g.add_card_to_hand(0, catalog::sheltered_by_ghosts());
        ready(&mut g);
        cast_at(&mut g, aura, Target::Permanent(mine));
        drain_stack(&mut g);
        let cp = g.computed_permanent(mine).unwrap();
        assert_eq!((cp.power, cp.toughness), (3, 2), "+1/+0");
        assert!(cp.keywords.contains(&Keyword::Lifelink), "gains lifelink");
        assert!(g.battlefield_find(foe).is_none(), "opponent permanent exiled on ETB");
    }

    /// Ragged Playmate makes a small creature unblockable for the turn.
    #[test]
    fn ragged_playmate_grants_unblockable() {
        let mut g = two_player_game();
        let rp = g.add_card_to_battlefield(0, catalog::ragged_playmate());
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2 (power 2)
        g.clear_sickness(rp);
        ready(&mut g);
        g.perform_action(GameAction::ActivateAbility {
            card_id: rp, ability_index: 0, target: Some(Target::Permanent(bear)),
            additional_targets: vec![], x_value: None,
        })
        .expect("activate Ragged Playmate");
        drain_stack(&mut g);
        assert!(g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Unblockable));
    }

    /// Hand That Feeds gets +2/+0 and menace on attack only with delirium active.
    #[test]
    fn hand_that_feeds_delirium_attack_buff() {
        let mut g = two_player_game();
        let hand = g.add_card_to_battlefield(0, catalog::hand_that_feeds());
        g.clear_sickness(hand);
        // Stock the graveyard with four card types for delirium.
        for c in [
            catalog::grizzly_bears(),       // creature
            catalog::lightning_bolt(),      // instant
            catalog::island(),             // land
            catalog::ornithopter(),        // artifact
        ] {
            let id = g.next_id();
            g.players[0].graveyard.push(crabomination::card::CardInstance::new(id, c, 0));
        }
        advance_to(&mut g, TurnStep::DeclareAttackers);
        g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker: hand, target: AttackTarget::Player(1),
        }]))
        .expect("attack");
        drain_stack(&mut g);
        let cp = g.computed_permanent(hand).unwrap();
        assert_eq!(cp.power, 4, "+2/+0 from delirium");
        assert!(cp.keywords.contains(&Keyword::Menace), "gains menace");
    }

    /// Marauding Dreadship incubates 2 on ETB.
    #[test]
    fn marauding_dreadship_etb_incubates() {
        let mut g = two_player_game();
        let ship = g.add_card_to_battlefield(0, catalog::marauding_dreadship());
        g.fire_self_etb_triggers(ship, 0);
        drain_stack(&mut g);
        assert_eq!(count_named(&g, 0, "Incubator"), 1, "Incubator token created");
    }

    /// Live or Die mode 0 reanimates a creature card from your graveyard.
    #[test]
    fn live_or_die_reanimates() {
        let mut g = two_player_game();
        let dead = g.next_id();
        g.players[0].graveyard.push(crabomination::card::CardInstance::new(dead, catalog::grizzly_bears(), 0));
        let lod = g.add_card_to_hand(0, catalog::live_or_die());
        ready(&mut g);
        g.perform_action(GameAction::CastSpell {
            card_id: lod, target: Some(Target::Permanent(dead)), additional_targets: vec![],
            mode: Some(0), x_value: None,
        })
        .expect("cast Live or Die (reanimate)");
        drain_stack(&mut g);
        assert!(g.battlefield.iter().any(|c| c.id == dead), "creature reanimated");
    }

    /// Unsettling Twins manifests dread on ETB.
    #[test]
    fn unsettling_twins_manifests_dread() {
        let mut g = two_player_game();
        for _ in 0..2 {
            g.add_card_to_library(0, catalog::grizzly_bears());
        }
        let ut = g.add_card_to_battlefield(0, catalog::unsettling_twins());
        g.fire_self_etb_triggers(ut, 0);
        drain_stack(&mut g);
        assert!(
            g.battlefield.iter().any(|c| c.controller == 0 && c.face_down),
            "manifest dread placed a face-down creature",
        );
    }

    /// Intruding Soulrager sacrifices a Room to deal 2 to each opponent and draw.
    #[test]
    fn intruding_soulrager_sacs_room() {
        let mut g = two_player_game();
        let sr = g.add_card_to_battlefield(0, catalog::intruding_soulrager());
        // A fully-cast Room on the battlefield as sac fodder.
        let room = g.add_card_to_battlefield(0, catalog::unholy_annex_ritual_chamber());
        g.clear_sickness(sr);
        g.add_card_to_library(0, catalog::grizzly_bears());
        ready(&mut g);
        let (foe_life, hand) = (g.players[1].life, g.players[0].hand.len());
        g.perform_action(GameAction::ActivateAbility {
            card_id: sr, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
        })
        .expect("activate Soulrager");
        drain_stack(&mut g);
        assert!(g.battlefield_find(room).is_none(), "Room sacrificed");
        assert_eq!(g.players[1].life, foe_life - 2, "2 damage to opponent");
        assert_eq!(g.players[0].hand.len(), hand + 1, "drew a card");
    }

    /// Clammy Prowler's attack trigger makes another attacker unblockable.
    #[test]
    fn clammy_prowler_makes_ally_unblockable() {
        let mut g = two_player_game();
        let cp = g.add_card_to_battlefield(0, catalog::clammy_prowler());
        let ally = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.clear_sickness(cp);
        g.clear_sickness(ally);
        advance_to(&mut g, TurnStep::DeclareAttackers);
        g.perform_action(GameAction::DeclareAttackers(vec![
            Attack { attacker: cp, target: AttackTarget::Player(1) },
            Attack { attacker: ally, target: AttackTarget::Player(1) },
        ]))
        .expect("attack");
        drain_stack(&mut g);
        assert!(
            g.computed_permanent(ally).unwrap().keywords.contains(&Keyword::Unblockable),
            "ally attacker became unblockable",
        );
    }

    /// Arabella drains each opponent for the number of your power-2-or-less creatures.
    #[test]
    fn arabella_attack_drains_for_small_creatures() {
        let mut g = two_player_game();
        let ara = g.add_card_to_battlefield(0, catalog::arabella_abandoned_doll());
        g.clear_sickness(ara);
        // Two more small creatures (Arabella herself is power 1, so 3 total ≤2 power).
        g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
        g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
        let (foe_life, my_life) = (g.players[1].life, g.players[0].life);
        advance_to(&mut g, TurnStep::DeclareAttackers);
        g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker: ara, target: AttackTarget::Player(1),
        }]))
        .expect("attack");
        drain_stack(&mut g);
        assert_eq!(g.players[1].life, foe_life - 3, "X=3 damage to opponent");
        assert_eq!(g.players[0].life, my_life + 3, "gained X=3 life");
    }

    /// Vile Mutilator forces each opponent to sacrifice an enchantment then a creature.
    #[test]
    fn vile_mutilator_double_edict() {
        let mut g = two_player_game();
        let foe_ench = g.add_card_to_battlefield(1, catalog::grasping_longneck()); // enchantment creature
        let foe_cre = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        // Pay the additional sacrifice with a spare creature.
        g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let vm = g.add_card_to_battlefield(0, catalog::vile_mutilator());
        g.fire_self_etb_triggers(vm, 0);
        drain_stack(&mut g);
        // Grasping Longneck is an enchantment creature → sacrificed by the first edict;
        // the plain bear by the second.
        assert!(g.battlefield_find(foe_ench).is_none(), "opponent's enchantment sacrificed");
        assert!(g.battlefield_find(foe_cre).is_none(), "opponent's creature sacrificed");
    }

    /// Disturbing Mirth draws two when you sacrifice a permanent to its ETB.
    #[test]
    fn disturbing_mirth_etb_may_sacrifice_draws() {
        use crabomination::decision::{DecisionAnswer, ScriptedDecider};
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::grizzly_bears()); // sac fodder
        for _ in 0..2 {
            g.add_card_to_library(0, catalog::island());
        }
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
        let dm = g.add_card_to_battlefield(0, catalog::disturbing_mirth());
        let hand = g.players[0].hand.len();
        g.fire_self_etb_triggers(dm, 0);
        drain_stack(&mut g);
        assert_eq!(g.players[0].hand.len(), hand + 2, "drew two from the sacrifice");
    }

    /// Synapse Necromage makes two can't-block Fungus tokens when it dies.
    #[test]
    fn synapse_necromage_dies_makes_fungus() {
        let mut g = two_player_game();
        let sn = g.add_card_to_battlefield(0, catalog::synapse_necromage());
        let mut evs = g.remove_to_graveyard_with_triggers(sn);
        evs.push(GameEvent::CreatureDied { card_id: sn });
        g.dispatch_triggers_for_events(&evs);
        drain_stack(&mut g);
        let fungi: Vec<_> = g.battlefield.iter().filter(|c| c.definition.name == "Fungus").collect();
        assert_eq!(fungi.len(), 2, "two Fungus tokens");
        assert!(fungi.iter().all(|c| c.definition.keywords.contains(&Keyword::CantBlock)));
    }

    /// Midnight Mayhem makes three Gremlins and grants them haste/menace/lifelink.
    #[test]
    fn midnight_mayhem_makes_and_buffs_gremlins() {
        let mut g = two_player_game();
        let mm = g.add_card_to_hand(0, catalog::midnight_mayhem());
        ready(&mut g);
        g.perform_action(GameAction::CastSpell {
            card_id: mm, target: None, additional_targets: vec![], mode: None, x_value: None,
        })
        .expect("cast Midnight Mayhem");
        drain_stack(&mut g);
        let gremlins: Vec<_> = g.battlefield.iter().filter(|c| c.definition.name == "Gremlin").map(|c| c.id).collect();
        assert_eq!(gremlins.len(), 3, "three Gremlins");
        let cp = g.computed_permanent(gremlins[0]).unwrap();
        assert!(cp.keywords.contains(&Keyword::Haste) && cp.keywords.contains(&Keyword::Menace) && cp.keywords.contains(&Keyword::Lifelink));
    }

    /// Stalked Researcher's Eerie trigger lets it attack despite defender.
    #[test]
    fn stalked_researcher_eerie_lifts_defender() {
        let mut g = two_player_game();
        let sr = g.add_card_to_battlefield(0, catalog::stalked_researcher());
        let room = g.add_card_to_hand(0, catalog::unholy_annex_ritual_chamber());
        g.clear_sickness(sr);
        ready(&mut g);
        g.perform_action(GameAction::CastRoomDoor { card_id: room, right: false })
            .expect("cast Room (enchantment enters → Eerie)");
        drain_stack(&mut g);
        advance_to(&mut g, TurnStep::DeclareAttackers);
        let res = g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker: sr, target: AttackTarget::Player(1),
        }]));
        assert!(res.is_ok(), "defender lifted by Eerie → attack legal");
    }

    /// Glimmer Bairn sacrifices a token for +2/+2.
    #[test]
    fn glimmer_bairn_sacs_token_to_pump() {
        let mut g = two_player_game();
        let gb = g.add_card_to_battlefield(0, catalog::glimmer_bairn());
        g.add_token_to_battlefield(0, &soldier_1_1());
        g.clear_sickness(gb);
        ready(&mut g);
        g.perform_action(GameAction::ActivateAbility {
            card_id: gb, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
        })
        .expect("activate Glimmer Bairn");
        drain_stack(&mut g);
        assert_eq!(g.computed_permanent(gb).unwrap().power, 3, "+2/+2 → 3/4");
    }

    /// Dashing Bloodsucker's Eerie trigger grants +2/+0 and lifelink.
    #[test]
    fn dashing_bloodsucker_eerie_pumps() {
        let mut g = two_player_game();
        let db = g.add_card_to_battlefield(0, catalog::dashing_bloodsucker());
        let room = g.add_card_to_hand(0, catalog::unholy_annex_ritual_chamber());
        ready(&mut g);
        g.perform_action(GameAction::CastRoomDoor { card_id: room, right: false })
            .expect("cast Room");
        drain_stack(&mut g);
        let cp = g.computed_permanent(db).unwrap();
        assert_eq!(cp.power, 4, "+2/+0 → 4/5");
        assert!(cp.keywords.contains(&Keyword::Lifelink));
    }

    /// Tunnel Surveyor makes a Glimmer token on ETB.
    #[test]
    fn tunnel_surveyor_makes_glimmer() {
        let mut g = two_player_game();
        let ts = g.add_card_to_battlefield(0, catalog::tunnel_surveyor());
        g.fire_self_etb_triggers(ts, 0);
        drain_stack(&mut g);
        assert_eq!(count_named(&g, 0, "Glimmer"), 1);
    }

    /// Pestilent Syphoner ships with flying and toxic 1.
    #[test]
    fn pestilent_syphoner_has_toxic() {
        let g = two_player_game();
        let def = catalog::pestilent_syphoner();
        assert!(def.keywords.contains(&Keyword::Flying));
        assert!(def.keywords.contains(&Keyword::Toxic(1)));
        let _ = g;
    }
}

mod recent25 {
    use crabomination::catalog;
    use crabomination::card::{CounterType, Keyword};
    use crabomination::game::types::{Attack, AttackTarget, Target};
    use crabomination::game::*;
    use crabomination::TurnStep;

    fn advance_to(g: &mut GameState, step: TurnStep) {
        while g.step != step {
            g.perform_action(GameAction::PassPriority).expect("pass priority");
        }
    }

    /// Sacrifice a battlefield permanent, firing dies triggers (CR 701.16).
    fn kill(g: &mut GameState, id: CardId) {
        let ctl = g.battlefield_find(id).unwrap().controller;
        let ctx = crabomination::game::effects::EffectContext::for_ability(id, ctl, Some(Target::Permanent(id)));
        g.resolve_effect(
            &crabomination::effect::Effect::SacrificePermanent { what: crabomination::effect::Selector::Target(0) },
            &ctx,
        )
        .unwrap();
        drain_stack(g);
    }

    /// Attack unblocked with `atk` at player 1, dealing combat damage.
    fn attack_unblocked(g: &mut GameState, atk: CardId) {
        g.clear_sickness(atk);
        advance_to(g, TurnStep::DeclareAttackers);
        g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker: atk,
            target: AttackTarget::Player(1),
        }]))
        .expect("declare attackers");
        drain_stack(g);
        g.step = TurnStep::CombatDamage;
        g.resolve_combat().expect("resolve combat");
        drain_stack(g);
    }

    /// Fear of Failed Tests draws cards equal to combat damage dealt to a player.
    #[test]
    fn fear_of_failed_tests_draws_on_hit() {
        let mut g = two_player_game();
        let fft = g.add_card_to_battlefield(0, catalog::fear_of_failed_tests()); // 2 power
        g.add_card_to_library(0, catalog::grizzly_bears());
        g.add_card_to_library(0, catalog::grizzly_bears());
        let before = g.players[0].hand.len();
        attack_unblocked(&mut g, fft);
        assert_eq!(g.players[0].hand.len(), before + 2, "drew 2 = combat damage");
    }

    /// Fear of Surveillance has vigilance and stays untapped when it attacks.
    #[test]
    fn fear_of_surveillance_vigilant() {
        let mut g = two_player_game();
        let fos = g.add_card_to_battlefield(0, catalog::fear_of_surveillance());
        g.add_card_to_library(0, catalog::grizzly_bears());
        assert!(catalog::fear_of_surveillance().keywords.contains(&Keyword::Vigilance));
        attack_unblocked(&mut g, fos);
        assert!(!g.battlefield_find(fos).unwrap().tapped, "vigilance keeps it untapped");
    }

    /// Fear of Being Hunted ships with haste and must-be-blocked.
    #[test]
    fn fear_of_being_hunted_keywords() {
        let def = catalog::fear_of_being_hunted();
        assert!(def.keywords.contains(&Keyword::Haste));
        assert!(def.keywords.contains(&Keyword::MustBeBlocked));
    }

    /// Fear of Immobility taps and stuns an opponent's creature on ETB.
    #[test]
    fn fear_of_immobility_taps_and_stuns() {
        let mut g = two_player_game();
        let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let fi = g.add_card_to_battlefield(0, catalog::fear_of_immobility());
        g.fire_self_etb_triggers(fi, 0);
        drain_stack(&mut g);
        let c = g.battlefield_find(foe).unwrap();
        assert!(c.tapped, "opponent creature tapped");
        assert!(c.counters.get(&CounterType::Stun).copied().unwrap_or(0) >= 1, "stun counter added");
    }

    /// Flesh Burrower grants deathtouch to another of your creatures when it attacks.
    #[test]
    fn flesh_burrower_grants_deathtouch_on_attack() {
        let mut g = two_player_game();
        let fb = g.add_card_to_battlefield(0, catalog::flesh_burrower());
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.clear_sickness(bear);
        attack_unblocked(&mut g, fb);
        assert!(
            g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Deathtouch),
            "ally gained deathtouch"
        );
    }

    /// Hardened Escort pumps and grants indestructible to another attacker.
    #[test]
    fn hardened_escort_pumps_ally() {
        let mut g = two_player_game();
        let he = g.add_card_to_battlefield(0, catalog::hardened_escort());
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
        g.clear_sickness(bear);
        attack_unblocked(&mut g, he);
        let cp = g.computed_permanent(bear).unwrap();
        assert_eq!(cp.power, 3, "+1/+0 ally");
        assert!(cp.keywords.contains(&Keyword::Indestructible));
    }

    /// Infernal Phantom deals damage equal to its power to a player when it dies.
    #[test]
    fn infernal_phantom_pings_on_death() {
        let mut g = two_player_game();
        let ip = g.add_card_to_battlefield(0, catalog::infernal_phantom()); // power 2
        let before = g.players[1].life;
        // Sacrifice to fire the dies trigger; auto-target hits the opponent's face.
        kill(&mut g, ip);
        assert_eq!(g.players[1].life, before - 2, "dies ping = power 2");
    }

    /// Lionheart Glimmer ships with Ward {2} and pumps the team when you attack.
    #[test]
    fn lionheart_glimmer_team_pump() {
        let mut g = two_player_game();
        let lg = g.add_card_to_battlefield(0, catalog::lionheart_glimmer());
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.clear_sickness(bear);
        assert!(matches!(catalog::lionheart_glimmer().keywords[0], Keyword::Ward(_)));
        attack_unblocked(&mut g, lg);
        assert_eq!(g.computed_permanent(bear).unwrap().power, 3, "team +1/+1 → 3 power");
    }

    /// Anthropede destroys a target Room when you pay {2} on ETB.
    #[test]
    fn anthropede_destroys_room() {
        let mut g = two_player_game();
        let room = g.add_card_to_battlefield(1, catalog::unholy_annex_ritual_chamber());
        let ant = g.add_card_to_battlefield(0, catalog::anthropede());
        g.players[0].mana_pool.add_colorless(2);
        g.decider = Box::new(crabomination::decision::ScriptedDecider::new([
            crabomination::decision::DecisionAnswer::Bool(true), // pay {2}
        ]));
        g.fire_self_etb_triggers(ant, 0);
        drain_stack(&mut g);
        assert!(g.battlefield_find(room).is_none(), "Room destroyed");
    }

    /// Living Phone digs for a small creature when it dies.
    #[test]
    fn living_phone_digs_on_death() {
        let mut g = two_player_game();
        let lp = g.add_card_to_battlefield(0, catalog::living_phone());
        g.add_card_to_library(0, catalog::grizzly_bears()); // 2/2 — power 2, eligible
        let hand_before = g.players[0].hand.len();
        kill(&mut g, lp);
        assert_eq!(g.players[0].hand.len(), hand_before + 1, "took the small creature");
    }

    /// Demonic Counsel tutors a Demon to hand (no delirium).
    #[test]
    fn demonic_counsel_finds_demon() {
        let mut g = two_player_game();
        let dc = g.add_card_to_hand(0, catalog::demonic_counsel());
        let demon = g.add_card_to_library(0, catalog::bloodgift_demon());
        g.players[0].mana_pool.add_colorless(1);
        g.players[0].mana_pool.add(crabomination::mana::Color::Black, 1);
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g.decider = Box::new(crabomination::decision::ScriptedDecider::new([
            crabomination::decision::DecisionAnswer::Search(Some(demon)),
        ]));
        g.perform_action(GameAction::CastSpell {
            card_id: dc, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast Demonic Counsel");
        drain_stack(&mut g);
        assert!(g.players[0].hand.iter().any(|c| c.id == demon), "Demon tutored to hand");
    }
}

mod recent26 {
    use crabomination::catalog;
    use crabomination::card::{CounterType, Keyword, Supertype};
    use crabomination::game::types::{Attack, AttackTarget};
    use crabomination::game::*;
    use crabomination::TurnStep;

    fn count_named(g: &GameState, controller: usize, name: &str) -> usize {
        g.battlefield
            .iter()
            .filter(|c| c.controller == controller && c.definition.name == name)
            .count()
    }

    /// Saddle `id` and attack player 1 with it, resolving the triggers.
    fn saddled_attack(g: &mut GameState, id: CardId) {
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

    /// Jibbirik Omnivore is a vanilla 3/2.
    #[test]
    fn jibbirik_omnivore_stats() {
        let def = catalog::jibbirik_omnivore();
        assert_eq!((def.power, def.toughness), (3, 2));
        assert!(def.activated_abilities.is_empty() && def.triggered_abilities.is_empty());
    }

    /// Caelorna is a legendary 0/8 wall.
    #[test]
    fn caelorna_is_legendary_wall() {
        let def = catalog::caelorna_coral_tyrant();
        assert_eq!((def.power, def.toughness), (0, 8));
        assert!(def.supertypes.contains(&Supertype::Legendary));
    }

    /// Gilded Ghoda makes a Treasure when it attacks while saddled.
    #[test]
    fn gilded_ghoda_makes_treasure_when_saddled() {
        let mut g = two_player_game();
        let gg = g.add_card_to_battlefield(0, catalog::gilded_ghoda());
        saddled_attack(&mut g, gg);
        assert_eq!(count_named(&g, 0, "Treasure"), 1, "saddled attack made a Treasure");
    }

    /// Brightfield Mustang untaps and grows when it attacks while saddled.
    #[test]
    fn brightfield_mustang_untaps_and_grows() {
        let mut g = two_player_game();
        let bm = g.add_card_to_battlefield(0, catalog::brightfield_mustang());
        saddled_attack(&mut g, bm);
        let c = g.battlefield_find(bm).unwrap();
        assert!(!c.tapped, "untapped by its own trigger after attacking");
        assert_eq!(g.computed_permanent(bm).unwrap().power, 4, "+1/+1 counter → 4 power");
    }

    /// Draconautics Engineer's first exhaust grants team haste and grows itself.
    #[test]
    fn draconautics_engineer_exhaust_haste() {
        let mut g = two_player_game();
        let de = g.add_card_to_battlefield(0, catalog::draconautics_engineer());
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.clear_sickness(de);
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g.players[0].mana_pool.add(crabomination::mana::Color::Red, 1);
        g.perform_action(GameAction::ActivateAbility {
            card_id: de, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
        }).expect("activate exhaust");
        drain_stack(&mut g);
        assert!(g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Haste), "ally got haste");
        assert_eq!(g.computed_permanent(de).unwrap().power, 3, "self +1/+1 → 3 power");
        // Exhaust: can't activate the same ability twice.
        g.players[0].mana_pool.add(crabomination::mana::Color::Red, 1);
        let again = g.perform_action(GameAction::ActivateAbility {
            card_id: de, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
        });
        assert!(again.is_err(), "exhaust ability is one-shot");
    }

    /// Afterburner Expert's exhaust puts two +1/+1 counters on it.
    #[test]
    fn afterburner_expert_exhaust_counters() {
        let mut g = two_player_game();
        let ae = g.add_card_to_battlefield(0, catalog::afterburner_expert());
        g.clear_sickness(ae);
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g.players[0].mana_pool.add_colorless(2);
        g.players[0].mana_pool.add(crabomination::mana::Color::Green, 2);
        g.perform_action(GameAction::ActivateAbility {
            card_id: ae, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
        }).expect("activate exhaust");
        drain_stack(&mut g);
        assert_eq!(g.battlefield_find(ae).unwrap().counters.get(&CounterType::PlusOnePlusOne).copied().unwrap_or(0), 2);
    }

    /// Piranha Fly ships with flying and an enters-tapped static.
    #[test]
    fn piranha_fly_flies_enters_tapped() {
        let def = catalog::piranha_fly();
        assert!(def.keywords.contains(&Keyword::Flying));
        assert!(def.static_abilities.iter().any(|s| matches!(
            s.effect,
            crabomination::card::StaticEffect::EntersTapped { .. }
        )));
    }

    /// Ripchain Razorkin sacrifices a land to draw a card.
    #[test]
    fn ripchain_razorkin_sacs_land_to_draw() {
        let mut g = two_player_game();
        let rr = g.add_card_to_battlefield(0, catalog::ripchain_razorkin());
        let land = g.add_card_to_battlefield(0, catalog::forest());
        g.add_card_to_library(0, catalog::grizzly_bears());
        g.clear_sickness(rr);
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g.players[0].mana_pool.add(crabomination::mana::Color::Red, 1);
        g.players[0].mana_pool.add_colorless(2);
        let hand = g.players[0].hand.len();
        g.perform_action(GameAction::ActivateAbility {
            card_id: rr, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
        }).expect("activate");
        drain_stack(&mut g);
        assert!(g.battlefield_find(land).is_none(), "land sacrificed");
        assert_eq!(g.players[0].hand.len(), hand + 1, "drew a card");
    }

    /// Beastrider Vanguard digs three for a permanent card.
    #[test]
    fn beastrider_vanguard_digs_for_permanent() {
        let mut g = two_player_game();
        let bv = g.add_card_to_battlefield(0, catalog::beastrider_vanguard());
        g.add_card_to_library(0, catalog::grizzly_bears()); // a permanent on top
        g.clear_sickness(bv);
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g.players[0].mana_pool.add(crabomination::mana::Color::Green, 1);
        g.players[0].mana_pool.add_colorless(4);
        let hand = g.players[0].hand.len();
        g.perform_action(GameAction::ActivateAbility {
            card_id: bv, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
        }).expect("activate");
        drain_stack(&mut g);
        assert_eq!(g.players[0].hand.len(), hand + 1, "took a permanent into hand");
    }

    /// Fear of Exposure's additional cost taps two of your creatures/lands.
    #[test]
    fn fear_of_exposure_taps_two_to_cast() {
        let mut g = two_player_game();
        let fear = g.add_card_to_hand(0, catalog::fear_of_exposure());
        let a = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let b = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.players[0].mana_pool.add(crabomination::mana::Color::Green, 1);
        g.players[0].mana_pool.add_colorless(2);
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::CastSpell {
            card_id: fear, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast Fear of Exposure");
        drain_stack(&mut g);
        assert!(g.battlefield_find(fear).is_some(), "Fear of Exposure resolved onto the battlefield");
        assert_eq!(
            [a, b].iter().filter(|&&id| g.battlefield_find(id).unwrap().tapped).count(),
            2, "two creatures tapped for the additional cost",
        );
    }

    /// Vicious Clown pumps itself when a small creature you control enters.
    #[test]
    fn vicious_clown_pumps_on_small_creature_etb() {
        let mut g = two_player_game();
        let clown = g.add_card_to_battlefield(0, catalog::vicious_clown());
        // A 2/2 (power ≤ 2) entering pumps the Clown +2/+0.
        let small = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.dispatch_triggers_for_events(&[GameEvent::PermanentEntered { card_id: small }]);
        drain_stack(&mut g);
        assert_eq!(g.computed_permanent(clown).unwrap().power, 4, "Clown pumped to 4 power");
        // A big creature (power > 2) does not trigger the pump.
        let big = g.add_card_to_battlefield(0, catalog::serra_angel()); // 4/4
        g.dispatch_triggers_for_events(&[GameEvent::PermanentEntered { card_id: big }]);
        drain_stack(&mut g);
        assert_eq!(g.computed_permanent(clown).unwrap().power, 4, "big creature did not pump further");
    }
}

mod recent27 {
    use crabomination::catalog;
    use crabomination::card::Keyword;
    use crabomination::game::types::{Attack, AttackTarget};
    use crabomination::game::*;
    use crabomination::TurnStep;

    fn count_named(g: &GameState, controller: usize, name: &str) -> usize {
        g.battlefield
            .iter()
            .filter(|c| c.controller == controller && c.definition.name == name)
            .count()
    }

    /// Brightblade Stoat is a 2/2 with first strike and lifelink.
    #[test]
    fn brightblade_stoat_keywords() {
        let mut g = two_player_game();
        let s = g.add_card_to_battlefield(0, catalog::brightblade_stoat());
        let cp = g.computed_permanent(s).unwrap();
        assert_eq!((cp.power, cp.toughness), (2, 2));
        assert!(cp.keywords.contains(&Keyword::FirstStrike) && cp.keywords.contains(&Keyword::Lifelink));
    }

    /// Pond Prophet draws a card on entry.
    #[test]
    fn pond_prophet_draws() {
        let mut g = two_player_game();
        g.add_card_to_library(0, catalog::grizzly_bears());
        let hand = g.players[0].hand.len();
        let p = g.move_card_to_battlefield_for_test(0, catalog::pond_prophet());
        drain_stack(&mut g);
        let _ = p;
        assert_eq!(g.players[0].hand.len(), hand + 1, "Pond Prophet drew a card");
    }

    /// Hecteyes makes each opponent discard on entry.
    #[test]
    fn hecteyes_discards_each_opponent() {
        let mut g = two_player_game();
        g.add_card_to_hand(1, catalog::grizzly_bears());
        let opp_hand = g.players[1].hand.len();
        g.move_card_to_battlefield_for_test(0, catalog::hecteyes());
        drain_stack(&mut g);
        assert_eq!(g.players[1].hand.len(), opp_hand - 1, "opponent discarded a card");
    }

    /// Agate-Blade Assassin drains 1 on attack.
    #[test]
    fn agate_blade_assassin_drains_on_attack() {
        let mut g = two_player_game();
        let a = g.add_card_to_battlefield(0, catalog::agate_blade_assassin());
        g.battlefield_find_mut(a).unwrap().summoning_sick = false;
        g.step = TurnStep::DeclareAttackers;
        g.priority.player_with_priority = 0;
        let (my, opp) = (g.players[0].life, g.players[1].life);
        g.declare_attackers(vec![Attack { attacker: a, target: AttackTarget::Player(1) }]).unwrap();
        drain_stack(&mut g);
        assert_eq!(g.players[1].life, opp - 1, "defender lost 1");
        assert_eq!(g.players[0].life, my + 1, "attacker gained 1");
    }

    /// Gigantoad is 4/4, pumping to 6/6 with seven lands.
    #[test]
    fn gigantoad_pumps_with_seven_lands() {
        let mut g = two_player_game();
        let toad = g.add_card_to_battlefield(0, catalog::gigantoad());
        assert_eq!(g.computed_permanent(toad).map(|c| (c.power, c.toughness)), Some((4, 4)));
        for _ in 0..7 { g.add_card_to_battlefield(0, catalog::forest()); }
        assert_eq!(g.computed_permanent(toad).map(|c| (c.power, c.toughness)), Some((6, 6)));
    }

    /// Loporrit Scout pumps itself when another creature enters.
    #[test]
    fn loporrit_scout_pumps_on_creature_etb() {
        let mut g = two_player_game();
        let scout = g.add_card_to_battlefield(0, catalog::loporrit_scout());
        let other = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.dispatch_triggers_for_events(&[GameEvent::PermanentEntered { card_id: other }]);
        drain_stack(&mut g);
        assert_eq!(g.computed_permanent(scout).unwrap().power, 4, "scout pumped +1/+1");
    }

    /// Head of the Homestead makes two Rabbit tokens on entry.
    #[test]
    fn head_of_the_homestead_makes_rabbits() {
        let mut g = two_player_game();
        g.move_card_to_battlefield_for_test(0, catalog::head_of_the_homestead());
        drain_stack(&mut g);
        assert_eq!(count_named(&g, 0, "Rabbit"), 2, "two Rabbit tokens");
    }

    /// Dwarven Castle Guard mints a Hero token when it dies.
    #[test]
    fn dwarven_castle_guard_dies_to_hero() {
        let mut g = two_player_game();
        let guard = g.add_card_to_battlefield(0, catalog::dwarven_castle_guard());
        let mut evs = Vec::new();
        g.sacrifice_one(guard, 0, &mut evs);
        g.dispatch_triggers_for_events(&evs);
        drain_stack(&mut g);
        assert_eq!(count_named(&g, 0, "Hero"), 1, "made a Hero token on death");
    }

    /// Shrike Force is a 1/3 flyer with double strike and vigilance.
    #[test]
    fn shrike_force_keywords() {
        let mut g = two_player_game();
        let s = g.add_card_to_battlefield(0, catalog::shrike_force());
        let cp = g.computed_permanent(s).unwrap();
        assert!(cp.keywords.contains(&Keyword::Flying));
        assert!(cp.keywords.contains(&Keyword::DoubleStrike));
        assert!(cp.keywords.contains(&Keyword::Vigilance));
    }

    /// Moonrise Cleric gains 1 life when it attacks.
    #[test]
    fn moonrise_cleric_gains_life_on_attack() {
        let mut g = two_player_game();
        let c = g.add_card_to_battlefield(0, catalog::moonrise_cleric());
        g.battlefield_find_mut(c).unwrap().summoning_sick = false;
        g.step = TurnStep::DeclareAttackers;
        g.priority.player_with_priority = 0;
        let my = g.players[0].life;
        g.declare_attackers(vec![Attack { attacker: c, target: AttackTarget::Player(1) }]).unwrap();
        drain_stack(&mut g);
        assert_eq!(g.players[0].life, my + 1, "gained 1 life on attack");
    }

    /// Dragoon's Wyvern mints a Hero token on entry.
    #[test]
    fn dragoons_wyvern_makes_hero() {
        let mut g = two_player_game();
        g.move_card_to_battlefield_for_test(0, catalog::dragoons_wyvern());
        drain_stack(&mut g);
        assert_eq!(count_named(&g, 0, "Hero"), 1, "made a Hero token on entry");
    }

    /// Coeurl taps a target nonenchantment creature.
    #[test]
    fn coeurl_taps_target_creature() {
        let mut g = two_player_game();
        let coeurl = g.add_card_to_battlefield(0, catalog::coeurl());
        g.battlefield_find_mut(coeurl).unwrap().summoning_sick = false;
        let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        g.players[0].mana_pool.add(crabomination::mana::Color::White, 1);
        g.players[0].mana_pool.add_colorless(1);
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::ActivateAbility {
            card_id: coeurl, ability_index: 0,
            target: Some(crabomination::game::types::Target::Permanent(victim)), additional_targets: vec![], x_value: None,
        }).expect("activate Coeurl");
        drain_stack(&mut g);
        assert!(g.battlefield_find(victim).unwrap().tapped, "target creature tapped");
    }

    /// Ahriman sacrifices another permanent to draw a card.
    #[test]
    fn ahriman_sacrifices_to_draw() {
        let mut g = two_player_game();
        let ahriman = g.add_card_to_battlefield(0, catalog::ahriman());
        g.add_card_to_battlefield(0, catalog::grizzly_bears()); // sac fodder
        g.add_card_to_library(0, catalog::grizzly_bears());
        g.players[0].mana_pool.add_colorless(3);
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        let hand = g.players[0].hand.len();
        g.perform_action(GameAction::ActivateAbility {
            card_id: ahriman, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
        }).expect("activate Ahriman");
        drain_stack(&mut g);
        assert_eq!(g.players[0].hand.len(), hand + 1, "drew a card off the sacrifice");
    }

    /// Gaelicat gets +2/+0 with two artifacts.
    #[test]
    fn gaelicat_pumps_with_artifacts() {
        let mut g = two_player_game();
        let cat = g.add_card_to_battlefield(0, catalog::gaelicat());
        assert_eq!(g.computed_permanent(cat).unwrap().power, 1);
        g.add_card_to_battlefield(0, catalog::sol_ring());
        g.add_card_to_battlefield(0, catalog::sol_ring());
        assert_eq!(g.computed_permanent(cat).unwrap().power, 3, "+2/+0 with two artifacts");
    }

    /// Scorpion Sentinel gets +3/+0 with seven lands.
    #[test]
    fn scorpion_sentinel_pumps_with_lands() {
        let mut g = two_player_game();
        let s = g.add_card_to_battlefield(0, catalog::scorpion_sentinel());
        assert_eq!(g.computed_permanent(s).unwrap().power, 1);
        for _ in 0..7 { g.add_card_to_battlefield(0, catalog::island()); }
        assert_eq!(g.computed_permanent(s).unwrap().power, 4, "1 +3 = 4 power");
    }

    /// Thistledown Players untaps a target nonland permanent on attack.
    #[test]
    fn thistledown_players_untaps_on_attack() {
        let mut g = two_player_game();
        let players = g.add_card_to_battlefield(0, catalog::thistledown_players());
        g.battlefield_find_mut(players).unwrap().summoning_sick = false;
        let ally = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.battlefield_find_mut(ally).unwrap().tapped = true;
        g.step = TurnStep::DeclareAttackers;
        g.priority.player_with_priority = 0;
        g.decider = Box::new(crabomination::decision::ScriptedDecider::new([
            crabomination::decision::DecisionAnswer::Target(crabomination::game::types::Target::Permanent(ally)),
        ]));
        g.declare_attackers(vec![Attack { attacker: players, target: AttackTarget::Player(1) }]).unwrap();
        drain_stack(&mut g);
        assert!(!g.battlefield_find(ally).unwrap().tapped, "ally untapped by the attack trigger");
    }

    /// Warren Elder pumps the team +1/+1 until end of turn.
    #[test]
    fn warren_elder_team_pump() {
        let mut g = two_player_game();
        let elder = g.add_card_to_battlefield(0, catalog::warren_elder());
        let buddy = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.players[0].mana_pool.add(crabomination::mana::Color::White, 1);
        g.players[0].mana_pool.add_colorless(3);
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::ActivateAbility {
            card_id: elder, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
        }).expect("activate Warren Elder");
        drain_stack(&mut g);
        assert_eq!(g.computed_permanent(buddy).unwrap().power, 3, "buddy pumped to 3");
        assert_eq!(g.computed_permanent(elder).unwrap().power, 3, "elder pumped to 3");
    }

    /// Jumbo Cactuar swings for +9999/+0 on attack.
    #[test]
    fn jumbo_cactuar_needles_on_attack() {
        let mut g = two_player_game();
        let cac = g.add_card_to_battlefield(0, catalog::jumbo_cactuar());
        g.battlefield_find_mut(cac).unwrap().summoning_sick = false;
        g.step = TurnStep::DeclareAttackers;
        g.priority.player_with_priority = 0;
        g.declare_attackers(vec![Attack { attacker: cac, target: AttackTarget::Player(1) }]).unwrap();
        drain_stack(&mut g);
        assert_eq!(g.computed_permanent(cac).unwrap().power, 10000, "1 + 9999 power");
    }

    /// Outlaw Medic draws on death.
    #[test]
    fn outlaw_medic_draws_on_death() {
        let mut g = two_player_game();
        let medic = g.add_card_to_battlefield(0, catalog::outlaw_medic());
        g.add_card_to_library(0, catalog::grizzly_bears());
        let hand = g.players[0].hand.len();
        let mut evs = Vec::new();
        g.sacrifice_one(medic, 0, &mut evs);
        g.dispatch_triggers_for_events(&evs);
        drain_stack(&mut g);
        assert_eq!(g.players[0].hand.len(), hand + 1, "drew a card on death");
    }

    /// Sterling Supplier counters another creature on entry.
    #[test]
    fn sterling_supplier_counters_ally() {
        let mut g = two_player_game();
        let ally = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.move_card_to_battlefield_for_test(0, catalog::sterling_supplier());
        drain_stack(&mut g);
        assert_eq!(
            g.battlefield_find(ally).unwrap().counter_count(crabomination::card::CounterType::PlusOnePlusOne),
            1, "ally got a +1/+1 counter",
        );
    }

    /// Shrieking Drake bounces a creature you control on entry.
    #[test]
    fn shrieking_drake_bounces_own() {
        let mut g = two_player_game();
        let ally = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.decider = Box::new(crabomination::decision::ScriptedDecider::new([
            crabomination::decision::DecisionAnswer::Target(crabomination::game::types::Target::Permanent(ally)),
        ]));
        g.move_card_to_battlefield_for_test(0, catalog::shrieking_drake());
        drain_stack(&mut g);
        assert!(g.battlefield_find(ally).is_none(), "ally bounced");
        assert!(g.players[0].hand.iter().any(|c| c.definition.name == "Grizzly Bears"), "ally in hand");
    }

    /// Oasis Gardener gains 2 life on entry and taps for any color.
    #[test]
    fn oasis_gardener_lifegain_and_mana() {
        let mut g = two_player_game();
        let life = g.players[0].life;
        let gard = g.move_card_to_battlefield_for_test(0, catalog::oasis_gardener());
        drain_stack(&mut g);
        assert_eq!(g.players[0].life, life + 2, "gained 2 life on entry");
        g.battlefield_find_mut(gard).unwrap().summoning_sick = false;
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::ActivateAbility {
            card_id: gard, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
        }).expect("tap for mana");
        assert!(g.players[0].mana_pool.total() >= 1, "added a mana");
    }

    /// Discerning Peddler loots (discard then draw) on entry.
    #[test]
    fn discerning_peddler_loots() {
        let mut g = two_player_game();
        g.add_card_to_hand(0, catalog::grizzly_bears());
        g.add_card_to_library(0, catalog::forest());
        let hand = g.players[0].hand.len();
        g.decider = Box::new(crabomination::decision::ScriptedDecider::new([
            crabomination::decision::DecisionAnswer::Bool(true),
        ]));
        g.move_card_to_battlefield_for_test(0, catalog::discerning_peddler());
        drain_stack(&mut g);
        assert_eq!(g.players[0].hand.len(), hand, "discarded one and drew one (net zero)");
    }
}

mod recent28 {
    use crabomination::catalog;
    use crabomination::card::Keyword;
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    use crabomination::game::types::{Attack, AttackTarget, Target};
    use crabomination::game::*;
    use crabomination::TurnStep;

    fn count_named(g: &GameState, controller: usize, name: &str) -> usize {
        g.battlefield
            .iter()
            .filter(|c| c.controller == controller && c.definition.name == name)
            .count()
    }

    /// Piggy Bank leaves a Treasure behind when it dies.
    #[test]
    fn piggy_bank_dies_makes_treasure() {
        let mut g = two_player_game();
        let pig = g.add_card_to_battlefield(0, catalog::piggy_bank());
        g.remove_to_graveyard_with_triggers(pig);
        drain_stack(&mut g);
        assert_eq!(count_named(&g, 0, "Treasure"), 1, "a Treasure on death");
    }

    /// Razorkin Hordecaller's "whenever you attack" fires once per combat,
    /// regardless of how many attackers are declared (CR 508, new YouAttack event).
    #[test]
    fn razorkin_you_attack_fires_once_per_combat() {
        let mut g = two_player_game();
        let raz = g.add_card_to_battlefield(0, catalog::razorkin_hordecaller());
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.clear_sickness(raz);
        g.clear_sickness(bear);
        g.active_player_idx = 0;
        g.step = TurnStep::DeclareAttackers;
        g.priority.player_with_priority = 0;
        g.declare_attackers(vec![
            Attack { attacker: raz, target: AttackTarget::Player(1) },
            Attack { attacker: bear, target: AttackTarget::Player(1) },
        ])
        .expect("attack");
        drain_stack(&mut g);
        assert_eq!(count_named(&g, 0, "Gremlin"), 1, "exactly one Gremlin for two attackers");
    }

    /// Appendage Amalgam has flash and a surveil-on-attack trigger.
    #[test]
    fn appendage_amalgam_flash() {
        let mut g = two_player_game();
        let a = g.add_card_to_battlefield(0, catalog::appendage_amalgam());
        let cp = g.computed_permanent(a).unwrap();
        assert_eq!((cp.power, cp.toughness), (3, 2));
        assert!(cp.keywords.contains(&Keyword::Flash));
    }

    /// Gremlin Tamer (recovered orphan) mints a Gremlin when an enchantment you
    /// control enters — the Eerie ability word.
    #[test]
    fn gremlin_tamer_eerie_makes_gremlin() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::gremlin_tamer());
        let ench = g.add_card_to_battlefield(0, catalog::sticky_fingers());
        g.dispatch_triggers_for_events(&[GameEvent::PermanentEntered { card_id: ench }]);
        drain_stack(&mut g);
        assert_eq!(count_named(&g, 0, "Gremlin"), 1, "Eerie made a Gremlin");
    }

    /// Shepherding Spirits plainscycles to fetch a Plains.
    #[test]
    fn shepherding_spirits_plainscycles() {
        let mut g = two_player_game();
        g.add_card_to_library(0, catalog::plains());
        g.add_card_to_library(0, catalog::grizzly_bears());
        let id = g.add_card_to_hand(0, catalog::shepherding_spirits());
        g.players[0].mana_pool.add_colorless(2);
        g.perform_action(GameAction::Landcycle { card_id: id }).expect("plainscycle");
        assert!(g.players[0].hand.iter().any(|c| c.definition.name == "Plains"), "fetched a Plains");
    }

    /// Seized from Slumber costs {3} less when it targets a tapped creature.
    #[test]
    fn seized_from_slumber_cheaper_vs_tapped() {
        let mut g = two_player_game();
        let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        g.battlefield.iter_mut().find(|c| c.id == foe).unwrap().tapped = true;
        let spell = g.add_card_to_hand(0, catalog::seized_from_slumber());
        g.active_player_idx = 0;
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        // {4}{W} − {3} = {1}{W} when the target is tapped.
        g.players[0].mana_pool.add(crabomination::mana::Color::White, 1);
        g.players[0].mana_pool.add_colorless(1);
        g.perform_action(GameAction::CastSpell {
            card_id: spell, target: Some(Target::Permanent(foe)), additional_targets: vec![],
            mode: None, x_value: None,
        }).expect("cast at reduced cost");
        drain_stack(&mut g);
        assert!(!g.battlefield.iter().any(|c| c.id == foe), "tapped creature destroyed");
    }

    /// Manifest Dread puts a face-down 2/2 onto the battlefield.
    #[test]
    fn manifest_dread_spell_makes_face_down() {
        let mut g = two_player_game();
        for _ in 0..3 {
            let id = g.next_id();
            g.players[0].add_to_library_top(id, catalog::grizzly_bears());
        }
        let spell = g.add_card_to_hand(0, catalog::manifest_dread_spell());
        g.active_player_idx = 0;
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g.players[0].mana_pool.add(crabomination::mana::Color::Green, 1);
        g.players[0].mana_pool.add_colorless(1);
        g.perform_action(GameAction::CastSpell {
            card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast Manifest Dread");
        drain_stack(&mut g);
        assert!(g.battlefield.iter().any(|c| c.controller == 0 && c.face_down), "a face-down creature");
    }

    /// Impossible Inferno deals 6 to a creature and, with delirium, exiles the top
    /// card with a play permission.
    #[test]
    fn impossible_inferno_burns_and_impulses_with_delirium() {
        let mut g = two_player_game();
        let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        // Four card types in graveyard → delirium active.
        for c in [catalog::lightning_bolt(), catalog::grizzly_bears(), catalog::ornithopter(), catalog::sticky_fingers()] {
            let id = g.next_id();
            g.players[0].send_to_graveyard(crabomination::card::CardInstance::new(id, c, 0));
        }
        let topid = g.next_id();
        g.players[0].add_to_library_top(topid, catalog::mountain());
        let spell = g.add_card_to_hand(0, catalog::impossible_inferno());
        g.active_player_idx = 0;
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g.players[0].mana_pool.add(crabomination::mana::Color::Red, 1);
        g.players[0].mana_pool.add_colorless(4);
        g.perform_action(GameAction::CastSpell {
            card_id: spell, target: Some(Target::Permanent(foe)), additional_targets: vec![],
            mode: None, x_value: None,
        }).expect("cast");
        drain_stack(&mut g);
        assert!(!g.battlefield.iter().any(|c| c.id == foe), "6 damage killed the bear");
        assert!(g.exile.iter().any(|c| c.id == topid && c.may_play_until.is_some()), "delirium exiled top with may-play");
    }

    /// Break Down the Door's first mode exiles a target artifact.
    #[test]
    fn break_down_the_door_exiles_artifact() {
        let mut g = two_player_game();
        let art = g.add_card_to_battlefield(1, catalog::ornithopter());
        let spell = g.add_card_to_hand(0, catalog::break_down_the_door());
        g.active_player_idx = 0;
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g.players[0].mana_pool.add(crabomination::mana::Color::Green, 1);
        g.players[0].mana_pool.add_colorless(2);
        g.perform_action(GameAction::CastSpell {
            card_id: spell, target: Some(Target::Permanent(art)), additional_targets: vec![],
            mode: Some(0), x_value: None,
        }).expect("cast mode 0");
        drain_stack(&mut g);
        assert!(g.exile.iter().any(|c| c.id == art), "artifact exiled");
    }

    /// Found Footage sacrifices for surveil-2-then-draw.
    #[test]
    fn found_footage_sac_draws() {
        let mut g = two_player_game();
        for _ in 0..4 {
            let id = g.next_id();
            g.players[0].add_to_library_top(id, catalog::grizzly_bears());
        }
        let clue = g.add_card_to_battlefield(0, catalog::found_footage());
        g.active_player_idx = 0;
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g.players[0].mana_pool.add_colorless(2);
        let hand_before = g.players[0].hand.len();
        g.perform_action(GameAction::ActivateAbility {
            card_id: clue, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
        }).expect("activate");
        drain_stack(&mut g);
        assert_eq!(g.players[0].hand.len(), hand_before + 1, "drew a card");
        assert!(!g.battlefield.iter().any(|c| c.id == clue), "Clue sacrificed");
    }

    /// Fear of Lost Teeth (recovered orphan) pings and gains life on death.
    #[test]
    fn fear_of_lost_teeth_dies_pings() {
        let mut g = two_player_game();
        let f = g.add_card_to_battlefield(0, catalog::fear_of_lost_teeth());
        let start = g.players[0].life;
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Target(Target::Player(1))]));
        g.remove_to_graveyard_with_triggers(f);
        drain_stack(&mut g);
        assert_eq!(g.players[1].life, 20 - 1, "1 damage to opponent");
        assert_eq!(g.players[0].life, start + 1, "gained 1 life");
    }
}

mod recent29 {
    use crabomination::catalog;
    use crabomination::card::{CardId, CounterType};
    use crabomination::game::types::{Attack, AttackTarget, Target};
    use crabomination::game::*;
    use crabomination::mana::Color;
    use crabomination::TurnStep;

    fn count_named(g: &GameState, controller: usize, name: &str) -> usize {
        g.battlefield
            .iter()
            .filter(|c| c.controller == controller && c.definition.name == name)
            .count()
    }

    /// Stand player 0 at PreCombatMain with priority and a full mana pool.
    fn ready(g: &mut GameState) {
        g.active_player_idx = 0;
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        for _ in 0..10 {
            g.players[0].mana_pool.add_colorless(1);
        }
        for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
            g.players[0].mana_pool.add(c, 4);
        }
    }

    /// Enter a card under `player`'s control through the real ETB funnel so
    /// self-source ETB triggers and enters-with-counters fire.
    fn etb_bf(g: &mut GameState, player: usize, def: crabomination::card::CardDefinition) -> CardId {
        let id = g.move_card_to_battlefield_for_test(player, def);
        drain_stack(g);
        id
    }

    /// Delta Bloodflies drains each opponent on attack while you control a
    /// counter-bearing creature (exercises `WithAnyCounter`).
    #[test]
    fn delta_bloodflies_drains_with_counter() {
        let mut g = two_player_game();
        let delta = g.add_card_to_battlefield(0, catalog::delta_bloodflies());
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.battlefield_find_mut(bear).unwrap().add_counters(CounterType::PlusOnePlusOne, 1);
        g.clear_sickness(delta);
        g.active_player_idx = 0;
        g.step = TurnStep::DeclareAttackers;
        g.priority.player_with_priority = 0;
        let life = g.players[1].life;
        g.declare_attackers(vec![Attack { attacker: delta, target: AttackTarget::Player(1) }])
            .expect("attack");
        drain_stack(&mut g);
        assert_eq!(g.players[1].life, life - 1, "opponent lost 1 life");
    }

    /// Meticulous Artisan mints a Treasure on entry.
    #[test]
    fn meticulous_artisan_etb_treasure() {
        let mut g = two_player_game();
        etb_bf(&mut g, 0, catalog::meticulous_artisan());
        assert_eq!(count_named(&g, 0, "Treasure"), 1, "ETB Treasure");
    }

    /// Iridescent Tiger adds five mana on entry.
    #[test]
    fn iridescent_tiger_etb_mana() {
        let mut g = two_player_game();
        etb_bf(&mut g, 0, catalog::iridescent_tiger());
        assert_eq!(g.players[0].mana_pool.total(), 5, "WUBRG added");
    }

    /// Unburied Earthcarver sacrifices a creature to grow.
    #[test]
    fn unburied_earthcarver_sac_grows() {
        let mut g = two_player_game();
        let ue = g.add_card_to_battlefield(0, catalog::unburied_earthcarver());
        let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        ready(&mut g);
        g.perform_action(GameAction::ActivateAbility {
            card_id: ue,
            ability_index: 0,
            target: None,
            additional_targets: vec![],
            x_value: None,
        })
        .expect("activate");
        drain_stack(&mut g);
        assert!(g.battlefield_find(fodder).is_none(), "fodder sacrificed");
        assert_eq!(g.battlefield_find(ue).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
    }

    /// Unrooted Ancestor sacrifices a creature for indestructibility + a tap.
    #[test]
    fn unrooted_ancestor_indestructible() {
        let mut g = two_player_game();
        let ua = g.add_card_to_battlefield(0, catalog::unrooted_ancestor());
        g.add_card_to_battlefield(0, catalog::grizzly_bears());
        ready(&mut g);
        g.perform_action(GameAction::ActivateAbility {
            card_id: ua,
            ability_index: 0,
            target: None,
            additional_targets: vec![],
            x_value: None,
        })
        .expect("activate");
        drain_stack(&mut g);
        let cp = g.computed_permanent(ua).unwrap();
        assert!(cp.keywords.contains(&crabomination::card::Keyword::Indestructible), "gained indestructible");
        assert!(g.battlefield_find(ua).unwrap().tapped, "tapped itself");
    }

    /// Gurmag Rakshasa's ETB shrinks an opponent's creature and pumps yours.
    #[test]
    fn gurmag_rakshasa_etb_modal() {
        let mut g = two_player_game();
        let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2 → dies to -2/-2
        let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        etb_bf(&mut g, 0, catalog::gurmag_rakshasa());
        assert!(g.battlefield_find(foe).is_none(), "opponent's 2/2 shrank to 0/0 and died");
        let cp = g.computed_permanent(mine).unwrap();
        assert_eq!((cp.power, cp.toughness), (4, 4), "your creature got +2/+2");
    }

    /// Fleeting Effigy returns itself to hand at its controller's end step.
    #[test]
    fn fleeting_effigy_self_bounce() {
        let mut g = two_player_game();
        let fe = g.add_card_to_battlefield(0, catalog::fleeting_effigy());
        g.active_player_idx = 0;
        g.fire_step_triggers(TurnStep::End);
        drain_stack(&mut g);
        assert!(g.battlefield_find(fe).is_none(), "left the battlefield");
        assert!(g.players[0].hand.iter().any(|c| c.id == fe), "returned to hand");
    }

    /// Host of the Hereafter enters with two +1/+1 counters and relays them on
    /// death.
    #[test]
    fn host_of_the_hereafter_counters() {
        let mut g = two_player_game();
        let host = etb_bf(&mut g, 0, catalog::host_of_the_hereafter());
        let cp = g.computed_permanent(host).unwrap();
        assert_eq!((cp.power, cp.toughness), (4, 4), "entered as 4/4");
        let heir = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.remove_to_graveyard_with_triggers(host);
        drain_stack(&mut g);
        assert_eq!(
            g.battlefield_find(heir).unwrap().counter_count(CounterType::PlusOnePlusOne),
            2,
            "counters moved onto the other creature",
        );
    }

    /// Overwhelming Surge deals 3 to a creature (both modes run by default).
    #[test]
    fn overwhelming_surge_burns() {
        let mut g = two_player_game();
        let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let art = g.add_card_to_battlefield(1, catalog::mind_stone());
        let os = g.add_card_to_hand(0, catalog::overwhelming_surge());
        ready(&mut g);
        g.perform_action(GameAction::CastSpell {
            card_id: os,
            target: Some(Target::Permanent(foe)),
            additional_targets: vec![Target::Permanent(art)],
            mode: None,
            x_value: None,
        })
        .expect("cast");
        drain_stack(&mut g);
        assert!(g.battlefield_find(foe).is_none(), "3 damage killed the 2/2");
        assert!(g.battlefield_find(art).is_none(), "artifact destroyed");
    }

    /// Marshal of the Lost pumps a creature by the number of attackers.
    #[test]
    fn marshal_of_the_lost_attack_pump() {
        let mut g = two_player_game();
        let marshal = g.add_card_to_battlefield(0, catalog::marshal_of_the_lost());
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.clear_sickness(marshal);
        g.clear_sickness(bear);
        g.active_player_idx = 0;
        g.step = TurnStep::DeclareAttackers;
        g.priority.player_with_priority = 0;
        g.declare_attackers(vec![
            Attack { attacker: marshal, target: AttackTarget::Player(1) },
            Attack { attacker: bear, target: AttackTarget::Player(1) },
        ])
        .expect("attack");
        drain_stack(&mut g);
        // Two attackers → +2/+2 on the auto-picked target.
        let buffed = g.battlefield.iter().any(|c| {
            let cp = g.computed_permanent(c.id).unwrap();
            c.controller == 0 && (cp.power, cp.toughness) == (4, 4)
        });
        assert!(buffed, "a creature got +2/+2 from two attackers");
    }

    /// Embermouth Sentinel tutors a basic land to the top of the library.
    #[test]
    fn embermouth_sentinel_tutors_to_top() {
        let mut g = two_player_game();
        g.add_card_to_library(0, catalog::forest());
        etb_bf(&mut g, 0, catalog::embermouth_sentinel());
        assert_eq!(g.players[0].library[0].definition.name, "Forest", "basic on top");
    }

    /// Rainveil Rejuvenator taps for green equal to its power.
    #[test]
    fn rainveil_rejuvenator_mana() {
        let mut g = two_player_game();
        let rr = g.add_card_to_battlefield(0, catalog::rainveil_rejuvenator());
        g.clear_sickness(rr);
        g.active_player_idx = 0;
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::ActivateAbility {
            card_id: rr,
            ability_index: 0,
            target: None,
            additional_targets: vec![],
            x_value: None,
        })
        .expect("tap for mana");
        assert_eq!(g.players[0].mana_pool.amount(Color::Green), 2, "added two green");
    }

    /// Synchronized Charge distributes two +1/+1 counters onto your creature.
    #[test]
    fn synchronized_charge_distributes() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let sc = g.add_card_to_hand(0, catalog::synchronized_charge());
        ready(&mut g);
        g.perform_action(GameAction::CastSpell {
            card_id: sc,
            target: Some(Target::Permanent(bear)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .expect("cast");
        drain_stack(&mut g);
        assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne), 2);
    }

    /// Watcher of the Wayside mills the opponent and gains you life.
    #[test]
    fn watcher_of_the_wayside_etb() {
        let mut g = two_player_game();
        g.add_card_to_library(1, catalog::grizzly_bears());
        g.add_card_to_library(1, catalog::grizzly_bears());
        let life = g.players[0].life;
        let gy = g.players[1].graveyard.len();
        etb_bf(&mut g, 0, catalog::watcher_of_the_wayside());
        assert_eq!(g.players[0].life, life + 2, "gained 2 life");
        assert_eq!(g.players[1].graveyard.len(), gy + 2, "opponent milled 2");
    }

    /// Teeming Dragonstorm makes two Soldiers on entry.
    #[test]
    fn teeming_dragonstorm_makes_soldiers() {
        let mut g = two_player_game();
        etb_bf(&mut g, 0, catalog::teeming_dragonstorm());
        assert_eq!(count_named(&g, 0, "Soldier"), 2, "two 2/2 Soldiers");
    }

    /// Ainok Wayfarer mills three and takes a land into hand.
    #[test]
    fn ainok_wayfarer_grabs_land() {
        let mut g = two_player_game();
        g.add_card_to_library(0, catalog::forest());
        g.add_card_to_library(0, catalog::grizzly_bears());
        g.add_card_to_library(0, catalog::grizzly_bears());
        let hand = g.players[0].hand.len();
        etb_bf(&mut g, 0, catalog::ainok_wayfarer());
        assert_eq!(g.players[0].hand.len(), hand + 1, "took a land to hand");
        assert!(g.players[0].hand.iter().any(|c| c.definition.name == "Forest"), "the land");
    }

    /// Tersa Lightshatter loots on entry (draw then discard, graveyard grows).
    #[test]
    fn tersa_lightshatter_loots() {
        let mut g = two_player_game();
        g.add_card_to_library(0, catalog::grizzly_bears());
        g.add_card_to_hand(0, catalog::grizzly_bears()); // discard fodder
        let gy = g.players[0].graveyard.len();
        etb_bf(&mut g, 0, catalog::tersa_lightshatter());
        assert_eq!(g.players[0].graveyard.len(), gy + 1, "discarded one");
    }

    /// Temur Tawnyback loots on entry.
    #[test]
    fn temur_tawnyback_loots() {
        let mut g = two_player_game();
        g.add_card_to_library(0, catalog::grizzly_bears());
        g.add_card_to_hand(0, catalog::grizzly_bears()); // discard fodder
        let gy = g.players[0].graveyard.len();
        etb_bf(&mut g, 0, catalog::temur_tawnyback());
        assert_eq!(g.players[0].graveyard.len(), gy + 1, "discarded one");
    }

    /// Focus the Mind draws three and discards one on resolution.
    #[test]
    fn focus_the_mind_draws() {
        let mut g = two_player_game();
        for _ in 0..3 {
            g.add_card_to_library(0, catalog::grizzly_bears());
        }
        let fm = g.add_card_to_hand(0, catalog::focus_the_mind());
        ready(&mut g);
        let lib = g.players[0].library.len();
        g.perform_action(GameAction::CastSpell {
            card_id: fm,
            target: None,
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .expect("cast");
        drain_stack(&mut g);
        assert_eq!(g.players[0].library.len(), lib - 3, "drew three");
    }

    /// Sage of the Skies copies itself when it's your second spell of the turn.
    #[test]
    fn sage_of_the_skies_copies() {
        let mut g = two_player_game();
        let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
        let sage = g.add_card_to_hand(0, catalog::sage_of_the_skies());
        ready(&mut g);
        g.perform_action(GameAction::CastSpell {
            card_id: bear, target: None, additional_targets: vec![], mode: None, x_value: None,
        })
        .expect("first spell");
        drain_stack(&mut g);
        g.perform_action(GameAction::CastSpell {
            card_id: sage, target: None, additional_targets: vec![], mode: None, x_value: None,
        })
        .expect("second spell");
        drain_stack(&mut g);
        assert_eq!(count_named(&g, 0, "Sage of the Skies"), 2, "original + token copy");
    }
}

mod recent30 {
    use crabomination::catalog;
    use crabomination::card::{CardId, CounterType, Keyword};
    use crabomination::game::types::{Attack, AttackTarget, Target};
    use crabomination::game::*;
    use crabomination::TurnStep;

    fn count_named(g: &GameState, controller: usize, name: &str) -> usize {
        g.battlefield
            .iter()
            .filter(|c| c.controller == controller && c.definition.name == name)
            .count()
    }

    fn ready(g: &mut GameState) {
        g.active_player_idx = 0;
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        for _ in 0..10 {
            g.players[0].mana_pool.add_colorless(1);
        }
        for c in [crabomination::mana::Color::White, crabomination::mana::Color::Blue, crabomination::mana::Color::Black,
                  crabomination::mana::Color::Red, crabomination::mana::Color::Green]
        {
            g.players[0].mana_pool.add(c, 4);
        }
    }

    fn etb_bf(g: &mut GameState, player: usize, def: crabomination::card::CardDefinition) -> CardId {
        let id = g.move_card_to_battlefield_for_test(player, def);
        drain_stack(g);
        id
    }

    /// Burner Rocket pumps a creature you control and grants trample on entry.
    #[test]
    fn burner_rocket_etb_pump() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        etb_bf(&mut g, 0, catalog::burner_rocket());
        let cp = g.computed_permanent(bear).unwrap();
        assert_eq!((cp.power, cp.toughness), (4, 2), "+2/+0");
        assert!(cp.keywords.contains(&Keyword::Trample), "gained trample");
    }

    /// Broadcast Rambler makes a Thopter on entry.
    #[test]
    fn broadcast_rambler_etb_thopter() {
        let mut g = two_player_game();
        etb_bf(&mut g, 0, catalog::broadcast_rambler());
        assert_eq!(count_named(&g, 0, "Thopter"), 1, "one 1/1 Thopter");
    }

    /// Carrion Cruiser mills two and returns a creature card from the graveyard.
    #[test]
    fn carrion_cruiser_etb_recursion() {
        use crabomination::decision::{DecisionAnswer, ScriptedDecider};
        let mut g = two_player_game();
        let bear = g.add_card_to_library(0, catalog::grizzly_bears());
        g.add_card_to_library(0, catalog::grizzly_bears());
        // The "return a creature/Vehicle" pick is offered as up-to-1, so script it.
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Cards(vec![bear])]));
        let hand = g.players[0].hand.len();
        etb_bf(&mut g, 0, catalog::carrion_cruiser());
        assert_eq!(g.players[0].hand.len(), hand + 1, "returned a milled creature to hand");
        assert!(g.players[0].hand.iter().any(|c| c.id == bear), "the chosen creature");
    }

    /// Clamorous Ironclad is a Crew 3 Vehicle with menace.
    #[test]
    fn clamorous_ironclad_shape() {
        let mut g = two_player_game();
        let ci = g.add_card_to_battlefield(0, catalog::clamorous_ironclad());
        let cp = g.computed_permanent(ci).unwrap();
        assert!(cp.keywords.contains(&Keyword::Crew(3)));
        assert!(cp.keywords.contains(&Keyword::Menace));
    }

    /// Alacrian Jaguar pumps itself when it attacks while saddled.
    #[test]
    fn alacrian_jaguar_saddled_pump() {
        let mut g = two_player_game();
        let jag = g.add_card_to_battlefield(0, catalog::alacrian_jaguar());
        let helper = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.clear_sickness(jag);
        g.clear_sickness(helper);
        ready(&mut g);
        g.perform_action(GameAction::Saddle { mount: jag, creatures: vec![helper] }).expect("saddle");
        g.step = TurnStep::DeclareAttackers;
        g.priority.player_with_priority = 0;
        g.declare_attackers(vec![Attack { attacker: jag, target: AttackTarget::Player(1) }])
            .expect("attack");
        drain_stack(&mut g);
        let cp = g.computed_permanent(jag).unwrap();
        assert_eq!((cp.power, cp.toughness), (6, 6), "+2/+2 while saddled");
    }

    /// District Mascot enters as a 1/1 (a +1/+1 counter on a 0/0).
    #[test]
    fn district_mascot_enters_with_counter() {
        let mut g = two_player_game();
        let dm = etb_bf(&mut g, 0, catalog::district_mascot());
        let cp = g.computed_permanent(dm).unwrap();
        assert_eq!((cp.power, cp.toughness), (1, 1), "0/0 + a +1/+1 counter");
    }

    /// Bulwark Ox's sacrifice gives your counter-bearing creatures indestructible.
    #[test]
    fn bulwark_ox_sac_protects_counter_creatures() {
        let mut g = two_player_game();
        let ox = g.add_card_to_battlefield(0, catalog::bulwark_ox());
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.battlefield_find_mut(bear).unwrap().add_counters(CounterType::PlusOnePlusOne, 1);
        ready(&mut g);
        g.perform_action(GameAction::ActivateAbility {
            card_id: ox, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
        })
        .expect("sac ability");
        drain_stack(&mut g);
        let cp = g.computed_permanent(bear).unwrap();
        assert!(cp.keywords.contains(&Keyword::Indestructible), "counter-creature gained indestructible");
        assert!(cp.keywords.contains(&Keyword::Hexproof), "and hexproof");
    }

    /// Autarch Mammoth makes an Elephant on entry.
    #[test]
    fn autarch_mammoth_etb_elephant() {
        let mut g = two_player_game();
        etb_bf(&mut g, 0, catalog::autarch_mammoth());
        assert_eq!(count_named(&g, 0, "Elephant"), 1, "a 3/3 Elephant");
    }

    /// Elvish Refueler's Exhaust ability grows it once; a second use is illegal.
    #[test]
    fn elvish_refueler_exhaust_once() {
        let mut g = two_player_game();
        let er = g.add_card_to_battlefield(0, catalog::elvish_refueler());
        ready(&mut g);
        g.perform_action(GameAction::ActivateAbility {
            card_id: er, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
        })
        .expect("first exhaust");
        drain_stack(&mut g);
        assert_eq!(g.battlefield_find(er).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
        let second = g.perform_action(GameAction::ActivateAbility {
            card_id: er, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
        });
        assert!(second.is_err(), "exhaust can be used only once");
    }

    /// Endrider Catalyzer ships with Start your engines!; its mana ability is
    /// gated behind max speed (illegal at speed 0).
    #[test]
    fn endrider_catalyzer_max_speed_gate() {
        let mut g = two_player_game();
        let ec = g.add_card_to_battlefield(0, catalog::endrider_catalyzer());
        g.clear_sickness(ec);
        assert!(g.computed_permanent(ec).unwrap().keywords.contains(&Keyword::StartYourEngines));
        ready(&mut g);
        let res = g.perform_action(GameAction::ActivateAbility {
            card_id: ec, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
        });
        assert!(res.is_err(), "max-speed ability is illegal below speed 4");
    }

    /// Collision Course deals damage equal to your creature/Vehicle count.
    #[test]
    fn collision_course_burns_for_board() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.add_card_to_battlefield(0, catalog::grizzly_bears()); // three creatures → X=3
        let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
        let cc = g.add_card_to_hand(0, catalog::collision_course());
        ready(&mut g);
        g.perform_action(GameAction::CastSpell {
            card_id: cc,
            target: Some(Target::Permanent(foe)),
            additional_targets: vec![],
            mode: Some(0),
            x_value: None,
        })
        .expect("cast");
        drain_stack(&mut g);
        assert!(g.battlefield_find(foe).is_none(), "3 damage killed the 2/2");
    }

    /// Back on Track reanimates a creature and leaves a Pilot behind.
    #[test]
    fn back_on_track_reanimates() {
        let mut g = two_player_game();
        let dead = g.add_card_to_graveyard(0, catalog::grizzly_bears());
        let bt = g.add_card_to_hand(0, catalog::back_on_track());
        ready(&mut g);
        g.perform_action(GameAction::CastSpell {
            card_id: bt,
            target: Some(Target::Permanent(dead)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .expect("cast");
        drain_stack(&mut g);
        assert!(g.battlefield_find(dead).is_some(), "creature reanimated");
        assert_eq!(count_named(&g, 0, "Pilot"), 1, "a 1/1 Pilot");
    }

    /// Dredger's Insight mills four on entry and lets you take a card.
    #[test]
    fn dredgers_insight_etb_mill() {
        let mut g = two_player_game();
        for _ in 0..4 {
            g.add_card_to_library(0, catalog::grizzly_bears());
        }
        let hand = g.players[0].hand.len();
        etb_bf(&mut g, 0, catalog::dredgers_insight());
        assert_eq!(g.players[0].hand.len(), hand + 1, "took a creature from the milled cards");
    }

    /// Aether Syphon taps for a card.
    #[test]
    fn aether_syphon_draws() {
        let mut g = two_player_game();
        g.add_card_to_library(0, catalog::grizzly_bears());
        let asy = g.add_card_to_battlefield(0, catalog::aether_syphon());
        ready(&mut g);
        let hand = g.players[0].hand.len();
        g.perform_action(GameAction::ActivateAbility {
            card_id: asy, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
        })
        .expect("draw ability");
        drain_stack(&mut g);
        assert_eq!(g.players[0].hand.len(), hand + 1, "drew a card");
    }

    /// Alacrian Armory is an anthem: +0/+1 and vigilance for your creatures.
    #[test]
    fn alacrian_armory_anthem() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.add_card_to_battlefield(0, catalog::alacrian_armory());
        let cp = g.computed_permanent(bear).unwrap();
        assert_eq!((cp.power, cp.toughness), (2, 3), "+0/+1");
        assert!(cp.keywords.contains(&Keyword::Vigilance), "gained vigilance");
    }

    /// Dracosaur Auxiliary pings any target when it attacks while saddled.
    #[test]
    fn dracosaur_auxiliary_saddled_ping() {
        let mut g = two_player_game();
        let drac = g.add_card_to_battlefield(0, catalog::dracosaur_auxiliary());
        let helper1 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let helper2 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.clear_sickness(drac);
        g.clear_sickness(helper1);
        g.clear_sickness(helper2);
        ready(&mut g);
        g.perform_action(GameAction::Saddle { mount: drac, creatures: vec![helper1, helper2] })
            .expect("saddle 3 via two 2-power bears");
        g.step = TurnStep::DeclareAttackers;
        g.priority.player_with_priority = 0;
        let life = g.players[1].life;
        g.declare_attackers(vec![Attack { attacker: drac, target: AttackTarget::Player(1) }])
            .expect("attack");
        drain_stack(&mut g);
        assert_eq!(g.players[1].life, life - 2, "dealt 2 to the opponent");
    }

    /// Detention Chariot exiles an opponent's creature until it leaves.
    #[test]
    fn detention_chariot_exiles_until_leaves() {
        let mut g = two_player_game();
        let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let dc = etb_bf(&mut g, 0, catalog::detention_chariot());
        assert!(g.battlefield_find(foe).is_none(), "opponent's creature exiled");
        g.remove_to_graveyard_with_triggers(dc);
        drain_stack(&mut g);
        assert!(g.battlefield.iter().any(|c| c.definition.name == "Grizzly Bears" && c.controller == 1),
            "returns when the Chariot leaves");
    }

    /// Endrider Spikespitter ships with reach and Start your engines!
    #[test]
    fn endrider_spikespitter_shape() {
        let mut g = two_player_game();
        let es = g.add_card_to_battlefield(0, catalog::endrider_spikespitter());
        let cp = g.computed_permanent(es).unwrap();
        assert!(cp.keywords.contains(&Keyword::Reach));
        assert!(cp.keywords.contains(&Keyword::StartYourEngines));
    }
}
