//! Tests for recentN card batches 148-162 (merged from per-batch micro-files).

mod recent148 {
    use crabomination::catalog;
    use crabomination::game::*;
    use crabomination::game::two_player_game;

    /// Faebloom Trick makes two Faerie flyers and taps an opponent's creature.
    #[test]
    fn faebloom_trick_tokens_and_tap() {
        let mut g = two_player_game();
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        let enemy = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let id = g.add_card_to_hand(0, catalog::faebloom_trick());
        g.players[0].mana_pool.add(crabomination::mana::Color::Blue, 1);
        g.players[0].mana_pool.add_colorless(2);
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: Some(Target::Permanent(enemy)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast");
        drain_stack(&mut g);
        let faeries = g.battlefield.iter()
            .filter(|c| c.definition.name == "Faerie" && c.controller == 0).count();
        assert_eq!(faeries, 2, "two Faerie tokens");
        assert!(g.battlefield_find(enemy).unwrap().tapped, "opponent's creature tapped");
    }

    /// Popular Egotist's sacrifice trigger drains an opponent for 1.
    #[test]
    fn popular_egotist_sacrifice_drains() {
        let mut g = two_player_game();
        let egotist = g.add_card_to_battlefield(0, catalog::popular_egotist());
        let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.players[0].mana_pool.add(crabomination::mana::Color::Black, 1);
        g.players[0].mana_pool.add_colorless(1);
        let opp_life = g.players[1].life;
        let my_life = g.players[0].life;
        g.perform_action(GameAction::ActivateAbility {
            card_id: egotist,
            ability_index: 0,
            target: None,
            additional_targets: vec![],
            x_value: None, mode: None,
        })
        .expect("activate sac ability");
        drain_stack(&mut g);
        assert!(g.battlefield_find(fodder).is_none(), "fodder sacrificed");
        assert!(
            g.computed_permanent(egotist).unwrap().keywords.contains(&crabomination::card::Keyword::Indestructible),
            "gained indestructible",
        );
        assert_eq!(g.players[1].life, opp_life - 1, "opponent drained for 1");
        assert_eq!(g.players[0].life, my_life + 1, "you gained 1");
    }

    /// Fear of Impostors counters a spell on ETB.
    #[test]
    fn fear_of_impostors_counters_on_etb() {
        let mut g = two_player_game();
        g.active_player_idx = 1; // p1's turn so they can cast at sorcery speed
        g.step = TurnStep::PreCombatMain;
        let bear = g.add_card_to_hand(1, catalog::grizzly_bears());
        g.players[1].mana_pool.add(crabomination::mana::Color::Green, 2);
        g.priority.player_with_priority = 1;
        g.perform_action(GameAction::CastSpell {
            card_id: bear, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast bear");
        // Flash in Fear of Impostors; its ETB counters the bear.
        let fear = g.add_card_to_hand(0, catalog::fear_of_impostors());
        g.players[0].mana_pool.add(crabomination::mana::Color::Blue, 2);
        g.players[0].mana_pool.add_colorless(1);
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::CastSpell {
            card_id: fear, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("flash the Nightmare");
        drain_stack(&mut g);
        assert!(!g.battlefield.iter().any(|c| c.id == bear), "bear countered by ETB");
    }

    /// Overwhelmed Apprentice mills each opponent two on ETB.
    #[test]
    fn overwhelmed_apprentice_mills_opponents() {
        let mut g = two_player_game();
        for _ in 0..3 {
            g.add_card_to_library(1, catalog::grizzly_bears());
        }
        g.move_card_to_battlefield_for_test(0, catalog::overwhelmed_apprentice());
        drain_stack(&mut g);
        assert_eq!(g.players[1].graveyard.len(), 2, "opponent milled two");
    }

    /// Cursed Windbreaker manifests a creature and equips it, granting flying.
    #[test]
    fn cursed_windbreaker_manifests_and_equips() {
        let mut g = two_player_game();
        g.add_card_to_library(0, catalog::grizzly_bears());
        g.add_card_to_library(0, catalog::grizzly_bears());
        let eq = g.move_card_to_battlefield_for_test(0, catalog::cursed_windbreaker());
        drain_stack(&mut g);
        // A 2/2 manifested creature exists and the Equipment is attached to it.
        let host = g.battlefield_find(eq).unwrap().attached_to.expect("equipped to the manifest");
        assert!(
            g.computed_permanent(host).unwrap().keywords.contains(&crabomination::card::Keyword::Flying),
            "equipped creature has flying",
        );
    }
}

mod recent149 {
    use crabomination::card::{CounterType, Keyword};
    use crabomination::catalog;
    use crabomination::game::types::Target;
    use crabomination::game::*;
    use crabomination::game::two_player_game;

    fn fill_mana(g: &mut GameState) {
        for c in [
            crabomination::mana::Color::White,
            crabomination::mana::Color::Blue,
            crabomination::mana::Color::Black,
            crabomination::mana::Color::Red,
            crabomination::mana::Color::Green,
        ] {
            g.players[0].mana_pool.add(c, 8);
        }
        g.players[0].mana_pool.add_colorless(8);
    }

    /// Driftgloom Coyote exiles a small opposing creature until it leaves and grows.
    #[test]
    fn driftgloom_coyote_exiles_and_grows() {
        let mut g = two_player_game();
        let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2, power 2
        let coyote = g.move_card_to_battlefield_for_test(0, catalog::driftgloom_coyote());
        drain_stack(&mut g);
        assert!(g.battlefield_find(victim).is_none(), "the 2/2 was exiled");
        assert_eq!(
            g.battlefield_find(coyote).unwrap().counter_count(CounterType::PlusOnePlusOne),
            1,
            "power-2 exile grew the Coyote",
        );
        // Coyote leaves → the exiled creature returns.
        let _ = g.remove_to_graveyard_with_triggers(coyote);
        drain_stack(&mut g);
        assert!(g.battlefield.iter().any(|c| c.id == victim), "exiled creature returned");
    }

    /// Early Winter mode 0 exiles a target creature.
    #[test]
    fn early_winter_exiles_creature() {
        let mut g = two_player_game();
        let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let id = g.add_card_to_hand(0, catalog::early_winter());
        fill_mana(&mut g);
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: Some(Target::Permanent(victim)), additional_targets: vec![], mode: Some(0), x_value: None,
        }).expect("cast Early Winter mode 0");
        drain_stack(&mut g);
        assert!(g.battlefield_find(victim).is_none(), "creature exiled");
    }

    /// High Stride pumps +1/+3, grants reach, and untaps the target.
    #[test]
    fn high_stride_pumps_reach_untap() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.battlefield_find_mut(bear).unwrap().tapped = true;
        let id = g.add_card_to_hand(0, catalog::high_stride());
        fill_mana(&mut g);
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast High Stride");
        drain_stack(&mut g);
        let c = g.computed_permanent(bear).unwrap();
        assert_eq!((c.power, c.toughness), (3, 5), "+1/+3");
        assert!(c.keywords.contains(&Keyword::Reach), "gained reach");
        assert!(!g.battlefield_find(bear).unwrap().tapped, "untapped");
    }

    /// Playful Shove deals 1 to a creature and draws a card.
    #[test]
    fn playful_shove_pings_and_draws() {
        let mut g = two_player_game();
        let lion = g.add_card_to_battlefield(1, catalog::savannah_lions()); // 2/1
        g.add_card_to_library(0, catalog::forest());
        let id = g.add_card_to_hand(0, catalog::playful_shove());
        let hand_before = g.players[0].hand.len(); // spell in hand
        fill_mana(&mut g);
        g.step = TurnStep::PreCombatMain;
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: Some(Target::Permanent(lion)), additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast Playful Shove");
        drain_stack(&mut g);
        assert!(g.battlefield_find(lion).is_none(), "2/1 died to 1 damage");
        assert_eq!(g.players[0].hand.len(), hand_before, "cast one, drew one → net hand size steady");
    }

    /// Psychic Whorl makes the opponent discard two; surveil fires only with a Rat.
    #[test]
    fn psychic_whorl_discards_and_conditional_surveil() {
        let mut g = two_player_game();
        g.add_card_to_hand(1, catalog::forest());
        g.add_card_to_hand(1, catalog::forest());
        let id = g.add_card_to_hand(0, catalog::psychic_whorl());
        fill_mana(&mut g);
        g.step = TurnStep::PreCombatMain;
        let opp_hand = g.players[1].hand.len();
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast Psychic Whorl");
        drain_stack(&mut g);
        assert_eq!(g.players[1].hand.len(), opp_hand - 2, "opponent discarded two");
    }

    /// Reptilian Recruiter steals a small creature until end of turn (untapped, hasty).
    #[test]
    fn reptilian_recruiter_threatens_small_creature() {
        let mut g = two_player_game();
        let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // power 2
        g.battlefield_find_mut(victim).unwrap().tapped = true;
        g.move_card_to_battlefield_for_test(0, catalog::reptilian_recruiter());
        drain_stack(&mut g);
        let c = g.battlefield_find(victim).unwrap();
        assert_eq!(c.controller, 0, "gained control of the power-2 creature");
        assert!(!c.tapped, "untapped it");
        assert!(g.computed_permanent(victim).unwrap().keywords.contains(&Keyword::Haste), "hasty");
    }

    /// Raccoon Rallier's sorcery-speed tap grants a creature haste.
    #[test]
    fn raccoon_rallier_grants_haste() {
        let mut g = two_player_game();
        let rallier = g.add_card_to_battlefield(0, catalog::raccoon_rallier());
        g.clear_sickness(rallier);
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.active_player_idx = 0;
        g.step = TurnStep::PreCombatMain;
        g.perform_action(GameAction::ActivateAbility {
            card_id: rallier, ability_index: 0, target: Some(Target::Permanent(bear)), additional_targets: vec![], x_value: None, mode: None,
        }).expect("activate Raccoon Rallier");
        drain_stack(&mut g);
        assert!(g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Haste), "granted haste");
    }
}

mod recent150 {
    use crabomination::card::Keyword;
    use crabomination::catalog;
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    use crabomination::game::types::Target;
    use crabomination::game::*;
    use crabomination::game::two_player_game;

    fn fill_mana(g: &mut GameState) {
        for c in [
            crabomination::mana::Color::White,
            crabomination::mana::Color::Blue,
            crabomination::mana::Color::Black,
            crabomination::mana::Color::Red,
            crabomination::mana::Color::Green,
        ] {
            g.players[0].mana_pool.add(c, 8);
        }
        g.players[0].mana_pool.add_colorless(8);
    }

    /// Consuming Ashes exiles a target creature (and surveils when MV ≤ 3).
    #[test]
    fn consuming_ashes_exiles_creature() {
        let mut g = two_player_game();
        let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // MV 2
        let id = g.add_card_to_hand(0, catalog::consuming_ashes());
        fill_mana(&mut g);
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: Some(Target::Permanent(victim)), additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast Consuming Ashes");
        drain_stack(&mut g);
        assert!(g.battlefield_find(victim).is_none(), "creature exiled");
    }

    /// Failed Fording bounces a nonland permanent to its owner's hand.
    #[test]
    fn failed_fording_bounces_permanent() {
        let mut g = two_player_game();
        let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let id = g.add_card_to_hand(0, catalog::failed_fording());
        fill_mana(&mut g);
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: Some(Target::Permanent(victim)), additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast Failed Fording");
        drain_stack(&mut g);
        assert!(g.battlefield_find(victim).is_none(), "bounced off the battlefield");
        assert!(g.players[1].hand.iter().any(|c| c.id == victim), "returned to owner's hand");
    }

    /// Harrier Strix taps a permanent on entry.
    #[test]
    fn harrier_strix_taps_on_etb() {
        let mut g = two_player_game();
        let target = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        g.move_card_to_battlefield_for_test(0, catalog::harrier_strix());
        drain_stack(&mut g);
        assert!(g.battlefield_find(target).unwrap().tapped, "ETB tapped the target");
    }

    /// Irascible Wolverine exiles the top card and lets you play it this turn.
    #[test]
    fn irascible_wolverine_impulse_top() {
        let mut g = two_player_game();
        let top = g.next_id();
        g.players[0].add_to_library_top(top, catalog::grizzly_bears());
        g.move_card_to_battlefield_for_test(0, catalog::irascible_wolverine());
        drain_stack(&mut g);
        assert!(g.exile.iter().any(|c| c.id == top), "top card exiled for impulse play");
    }

    /// Killer's Mask manifests a dread creature and equips it (granting menace).
    #[test]
    fn killers_mask_manifest_and_equip() {
        let mut g = two_player_game();
        let id1 = g.next_id();
        g.players[0].add_to_library_top(id1, catalog::grizzly_bears());
        let id2 = g.next_id();
        g.players[0].add_to_library_top(id2, catalog::forest());
        let mask = g.move_card_to_battlefield_for_test(0, catalog::killers_mask());
        drain_stack(&mut g);
        // A 2/2 face-down manifest exists and the Equipment is attached to it.
        let manifest = g.battlefield.iter().find(|c| c.face_down && c.controller == 0).map(|c| c.id);
        assert!(manifest.is_some(), "manifested a face-down creature");
        let attached_to = g.battlefield_find(mask).unwrap().attached_to;
        assert_eq!(attached_to, manifest, "Equipment attached to the manifest");
        assert!(g.computed_permanent(manifest.unwrap()).unwrap().keywords.contains(&Keyword::Menace),
            "equipped creature has menace");
    }

    /// Jump Scare gives +2/+2 and flying until end of turn.
    #[test]
    fn jump_scare_pumps_and_flies() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let id = g.add_card_to_hand(0, catalog::jump_scare());
        fill_mana(&mut g);
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast Jump Scare");
        drain_stack(&mut g);
        let c = g.computed_permanent(bear).unwrap();
        assert_eq!((c.power, c.toughness), (4, 4), "+2/+2");
        assert!(c.keywords.contains(&Keyword::Flying), "gained flying");
    }

    /// Expel the Interlopers destroys creatures with power ≥ the chosen number.
    #[test]
    fn expel_the_interlopers_destroys_by_chosen_power() {
        let mut g = two_player_game();
        let big = g.add_card_to_battlefield(1, catalog::colossal_dreadmaw()); // 6/6
        let small = g.add_card_to_battlefield(0, catalog::savannah_lions());  // 2/1
        let id = g.add_card_to_hand(0, catalog::expel_the_interlopers());
        fill_mana(&mut g);
        // Choose 4 → only power-4+ creatures die.
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Amount(4)]));
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast Expel the Interlopers");
        drain_stack(&mut g);
        assert!(g.battlefield_find(big).is_none(), "power-6 creature destroyed");
        assert!(g.battlefield_find(small).is_some(), "power-2 creature survives the chosen 4");
    }
}

mod recent151 {
    use crabomination::card::{CounterType, Keyword};
    use crabomination::catalog;
    use crabomination::game::types::{Attack, AttackTarget, Target};
    use crabomination::game::*;
    use crabomination::game::two_player_game;
    use crabomination::mana::Color;

    fn fill_mana(g: &mut GameState) {
        for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
            g.players[0].mana_pool.add(c, 8);
        }
        g.players[0].mana_pool.add_colorless(8);
    }

    /// Resurrected Cultist reanimates itself from the graveyard with delirium, with
    /// a finality counter.
    #[test]
    fn resurrected_cultist_delirium_reanimate() {
        let mut g = two_player_game();
        let cultist = g.add_card_to_graveyard(0, catalog::resurrected_cultist());
        // Seed four card types in the graveyard for delirium.
        g.add_card_to_graveyard(0, catalog::forest()); // Land
        g.add_card_to_graveyard(0, catalog::lightning_strike()); // Instant
        g.add_card_to_graveyard(0, catalog::grizzly_bears()); // Creature (+ the Cultist)
        g.add_card_to_graveyard(0, catalog::sol_ring()); // Artifact
        g.active_player_idx = 0;
        g.step = crabomination::game::TurnStep::PreCombatMain;
        fill_mana(&mut g);
        g.perform_action(GameAction::ActivateAbility {
            card_id: cultist, ability_index: 0, target: None, additional_targets: vec![], x_value: None, mode: None,
        }).expect("delirium reanimate");
        drain_stack(&mut g);
        let c = g.battlefield_find(cultist).expect("returned to battlefield");
        assert_eq!(c.counter_count(CounterType::Finality), 1, "entered with a finality counter");
    }

    /// Overgrown Zealot taps for one mana of any color.
    #[test]
    fn overgrown_zealot_taps_for_any_color() {
        let mut g = two_player_game();
        let zealot = g.add_card_to_battlefield(0, catalog::overgrown_zealot());
        g.clear_sickness(zealot);
        g.perform_action(GameAction::ActivateAbility {
            card_id: zealot, ability_index: 0, target: None, additional_targets: vec![], x_value: None, mode: None,
        }).expect("tap for mana");
        assert!(g.players[0].mana_pool.total() >= 1, "produced a mana");
        assert!(g.battlefield_find(zealot).unwrap().tapped, "tapped for the mana");
    }

    /// Gila Courser impulses the top card when it attacks while saddled.
    #[test]
    fn gila_courser_saddled_attack_impulse() {
        let mut g = two_player_game();
        let courser = g.add_card_to_battlefield(0, catalog::gila_courser());
        g.clear_sickness(courser);
        let top = g.next_id();
        g.players[0].add_to_library_top(top, catalog::grizzly_bears());
        g.battlefield_find_mut(courser).unwrap().saddled = true;
        g.active_player_idx = 0;
        g.step = crabomination::game::TurnStep::DeclareAttackers;
        g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker: courser, target: AttackTarget::Player(1),
        }])).unwrap();
        drain_stack(&mut g);
        assert!(g.exile.iter().any(|c| c.id == top), "saddled attack impulsed the top card");
    }

    /// Grab the Prize draws two and burns each opponent when a nonland was discarded.
    #[test]
    fn grab_the_prize_nonland_discard_burns() {
        let mut g = two_player_game();
        g.add_card_to_hand(0, catalog::grizzly_bears()); // nonland to discard
        let id = g.add_card_to_hand(0, catalog::grab_the_prize());
        for _ in 0..3 {
            g.add_card_to_library(0, catalog::forest());
        }
        fill_mana(&mut g);
        g.step = TurnStep::PreCombatMain;
        let opp_life = g.players[1].life;
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast Grab the Prize");
        drain_stack(&mut g);
        assert_eq!(g.players[1].life, opp_life - 2, "nonland discard dealt 2 to the opponent");
    }

    /// Malevolent Chandelier bottoms a card from a graveyard.
    #[test]
    fn malevolent_chandelier_bottoms_graveyard_card() {
        let mut g = two_player_game();
        let chandelier = g.add_card_to_battlefield(0, catalog::malevolent_chandelier());
        g.clear_sickness(chandelier);
        let corpse = g.add_card_to_graveyard(1, catalog::grizzly_bears());
        g.active_player_idx = 0;
        g.step = TurnStep::PreCombatMain;
        fill_mana(&mut g);
        g.perform_action(GameAction::ActivateAbility {
            card_id: chandelier, ability_index: 0, target: Some(Target::Permanent(corpse)), additional_targets: vec![], x_value: None, mode: None,
        }).expect("bottom a graveyard card");
        drain_stack(&mut g);
        assert!(g.players[1].graveyard.iter().all(|c| c.id != corpse), "left the graveyard");
        assert!(g.players[1].library.iter().any(|c| c.id == corpse), "went to the library");
    }

    /// Moonstone Harbinger pumps your Bats when you gain life on your turn.
    #[test]
    fn moonstone_harbinger_life_gain_pumps_bats() {
        let mut g = two_player_game();
        let bat = g.add_card_to_battlefield(0, catalog::moonstone_harbinger());
        g.active_player_idx = 0;
        g.adjust_life(0, 1);
        g.dispatch_triggers_for_events(&[GameEvent::LifeGained { player: 0, amount: 1 }]);
        drain_stack(&mut g);
        let c = g.computed_permanent(bat).unwrap();
        assert_eq!(c.power, 2, "Bat got +1/+0 on lifegain");
        assert!(c.keywords.contains(&Keyword::Deathtouch), "still deathtouch");
    }
}

mod recent152 {
    use crabomination::card::CounterType;
    use crabomination::catalog;
    use crabomination::game::types::Target;
    use crabomination::game::*;
    use crabomination::game::two_player_game;
    use crabomination::mana::Color;

    fn fill_mana(g: &mut GameState) {
        for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
            g.players[0].mana_pool.add(c, 8);
        }
        g.players[0].mana_pool.add_colorless(8);
    }

    /// Rowan's Grim Search draws two and loses 2 life.
    #[test]
    fn rowans_grim_search_draws_and_loses() {
        let mut g = two_player_game();
        for _ in 0..4 {
            g.add_card_to_library(0, catalog::forest());
        }
        let id = g.add_card_to_hand(0, catalog::rowans_grim_search());
        let hand_before = g.players[0].hand.len();
        let life = g.players[0].life;
        fill_mana(&mut g);
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast Rowan's Grim Search");
        drain_stack(&mut g);
        // -1 (cast) + 2 (draw) = +1 net hand.
        assert_eq!(g.players[0].hand.len(), hand_before + 1, "drew two");
        assert_eq!(g.players[0].life, life - 2, "lost 2 life");
    }

    /// Rite of the Moth reanimates a graveyard creature with a finality counter.
    #[test]
    fn rite_of_the_moth_reanimates_with_finality() {
        let mut g = two_player_game();
        let dead = g.add_card_to_graveyard(0, catalog::grizzly_bears());
        let id = g.add_card_to_hand(0, catalog::rite_of_the_moth());
        fill_mana(&mut g);
        g.step = TurnStep::PreCombatMain;
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: Some(Target::Permanent(dead)), additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast Rite of the Moth");
        drain_stack(&mut g);
        let c = g.battlefield_find(dead).expect("reanimated to the battlefield");
        assert_eq!(c.counter_count(CounterType::Finality), 1, "with a finality counter");
    }

    /// Hazel's Nocturne returns up to two graveyard creatures and drains 2.
    #[test]
    fn hazels_nocturne_recurs_and_drains() {
        let mut g = two_player_game();
        let a = g.add_card_to_graveyard(0, catalog::grizzly_bears());
        let b = g.add_card_to_graveyard(0, catalog::savannah_lions());
        let id = g.add_card_to_hand(0, catalog::hazels_nocturne());
        fill_mana(&mut g);
        let opp_life = g.players[1].life;
        let my_life = g.players[0].life;
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast Hazel's Nocturne");
        drain_stack(&mut g);
        assert!(g.players[0].hand.iter().any(|c| c.id == a) && g.players[0].hand.iter().any(|c| c.id == b),
            "both creatures returned to hand");
        assert_eq!(g.players[1].life, opp_life - 2, "opponent lost 2");
        assert_eq!(g.players[0].life, my_life + 2, "you gained 2");
    }

    /// Form a Posse creates X Mercenary tokens.
    #[test]
    fn form_a_posse_makes_x_tokens() {
        let mut g = two_player_game();
        let id = g.add_card_to_hand(0, catalog::form_a_posse());
        fill_mana(&mut g);
        g.step = TurnStep::PreCombatMain;
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: None, additional_targets: vec![], mode: None, x_value: Some(3),
        }).expect("cast Form a Posse with X=3");
        drain_stack(&mut g);
        let mercs = g.battlefield.iter().filter(|c| c.definition.name == "Mercenary" && c.controller == 0).count();
        assert_eq!(mercs, 3, "made three Mercenary tokens");
    }

    /// Otterball Antics makes a prowess Otter (no counter when cast from hand).
    #[test]
    fn otterball_antics_makes_otter() {
        let mut g = two_player_game();
        let id = g.add_card_to_hand(0, catalog::otterball_antics());
        fill_mana(&mut g);
        g.step = TurnStep::PreCombatMain;
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast Otterball Antics");
        drain_stack(&mut g);
        let otter = g.battlefield.iter().find(|c| c.definition.name == "Otter" && c.controller == 0)
            .expect("made an Otter");
        assert_eq!(otter.counter_count(CounterType::PlusOnePlusOne), 0, "no counter when cast from hand");
        assert!(otter.definition.keywords.contains(&crabomination::card::Keyword::Prowess), "has prowess");
    }
}

mod recent153 {
    use crabomination::card::Keyword;
    use crabomination::catalog;
    use crabomination::game::*;
    use crabomination::game::two_player_game;
    use crabomination::mana::Color;

    fn fill_mana(g: &mut GameState) {
        for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
            g.players[0].mana_pool.add(c, 8);
        }
        g.players[0].mana_pool.add_colorless(8);
    }

    /// Gold Pan makes a Treasure on entry and buffs the creature it equips.
    #[test]
    fn gold_pan_makes_treasure_and_buffs() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let pan = g.move_card_to_battlefield_for_test(0, catalog::gold_pan());
        drain_stack(&mut g);
        assert!(g.battlefield.iter().any(|c| c.definition.name == "Treasure" && c.controller == 0),
            "ETB minted a Treasure");
        fill_mana(&mut g);
        g.step = TurnStep::PreCombatMain;
        g.perform_action(GameAction::Equip { equipment: pan, target: bear })
            .expect("equip Gold Pan");
        let c = g.computed_permanent(bear).unwrap();
        assert_eq!((c.power, c.toughness), (3, 3), "equipped creature +1/+1");
    }

    /// Conductive Machete manifests a dread creature and equips it (+2/+1).
    #[test]
    fn conductive_machete_manifest_and_equip() {
        let mut g = two_player_game();
        let id1 = g.next_id();
        g.players[0].add_to_library_top(id1, catalog::grizzly_bears());
        let id2 = g.next_id();
        g.players[0].add_to_library_top(id2, catalog::forest());
        let machete = g.move_card_to_battlefield_for_test(0, catalog::conductive_machete());
        drain_stack(&mut g);
        let manifest = g.battlefield.iter().find(|c| c.face_down && c.controller == 0).map(|c| c.id);
        assert!(manifest.is_some(), "manifested a face-down creature");
        assert_eq!(g.battlefield_find(machete).unwrap().attached_to, manifest, "attached to the manifest");
        let c = g.computed_permanent(manifest.unwrap()).unwrap();
        assert_eq!((c.power, c.toughness), (4, 3), "2/2 manifest +2/+1 = 4/3");
    }

    /// Baron Bertram Graywater mints a Vampire when a token you control enters.
    #[test]
    fn baron_makes_vampire_on_token_enter() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::baron_bertram_graywater());
        let tok = g.add_token_to_battlefield(0, &crabomination_base::tokens::treasure_token());
        g.dispatch_triggers_for_events(&[GameEvent::TokenCreated { card_id: tok }]);
        drain_stack(&mut g);
        assert!(g.battlefield.iter().any(|c| c.definition.name == "Vampire" && c.controller == 0),
            "a token entering minted a Vampire");
    }

    /// Jem Lightfoote draws at end step when you haven't cast a spell.
    #[test]
    fn jem_lightfoote_draws_when_spell_free() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::jem_lightfoote_sky_explorer());
        g.add_card_to_library(0, catalog::forest());
        g.active_player_idx = 0;
        let hand = g.players[0].hand.len();
        g.fire_step_triggers(TurnStep::End);
        drain_stack(&mut g);
        assert_eq!(g.players[0].hand.len(), hand + 1, "drew a card at end step (no spell cast)");
    }

    /// Canyon Crab's activated ability shifts it +2/-2.
    #[test]
    fn canyon_crab_pump_shifts_stats() {
        let mut g = two_player_game();
        let crab = g.add_card_to_battlefield(0, catalog::canyon_crab());
        g.clear_sickness(crab);
        fill_mana(&mut g);
        g.perform_action(GameAction::ActivateAbility {
            card_id: crab, ability_index: 0, target: None, additional_targets: vec![], x_value: None, mode: None,
        }).expect("activate Canyon Crab");
        drain_stack(&mut g);
        let c = g.computed_permanent(crab).unwrap();
        assert_eq!((c.power, c.toughness), (2, 3), "0/5 → 2/3 after +2/-2");
        assert!(!c.keywords.contains(&Keyword::Flying), "no flying (sanity)");
    }
}

mod recent154 {
    use crabomination::catalog;
    use crabomination::game::types::Target;
    use crabomination::game::*;
    use crabomination::game::two_player_game;
    use crabomination::mana::Color;

    fn fill_mana(g: &mut GameState) {
        for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
            g.players[0].mana_pool.add(c, 8);
        }
        g.players[0].mana_pool.add_colorless(8);
    }

    /// Harnesser of Storms impulses the top card when you cast a noncreature spell.
    #[test]
    fn harnesser_impulses_on_noncreature_cast() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::harnesser_of_storms());
        let top = g.next_id();
        g.players[0].add_to_library_top(top, catalog::grizzly_bears());
        let bolt = g.add_card_to_hand(0, catalog::lightning_strike());
        fill_mana(&mut g);
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        g.step = TurnStep::PreCombatMain;
        g.perform_action(GameAction::CastSpell {
            card_id: bolt, target: Some(Target::Player(1)), additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast a noncreature spell");
        drain_stack(&mut g);
        assert!(g.exile.iter().any(|c| c.id == top), "impulsed the top card on a noncreature cast");
    }

    /// Flamecache Gecko's rummage draws for a discard.
    #[test]
    fn flamecache_gecko_rummage() {
        let mut g = two_player_game();
        let gecko = g.add_card_to_battlefield(0, catalog::flamecache_gecko());
        g.clear_sickness(gecko);
        g.add_card_to_hand(0, catalog::forest()); // a card to discard
        g.add_card_to_library(0, catalog::island());
        fill_mana(&mut g);
        let hand = g.players[0].hand.len();
        g.perform_action(GameAction::ActivateAbility {
            card_id: gecko, ability_index: 0, target: None, additional_targets: vec![], x_value: None, mode: None,
        }).expect("rummage");
        drain_stack(&mut g);
        assert_eq!(g.players[0].hand.len(), hand, "discarded one, drew one");
    }

    /// Intimidation Campaign's ETB drains, gains, and draws.
    #[test]
    fn intimidation_campaign_etb_value() {
        let mut g = two_player_game();
        g.add_card_to_library(0, catalog::forest());
        let opp = g.players[1].life;
        let me = g.players[0].life;
        let hand = g.players[0].hand.len();
        g.move_card_to_battlefield_for_test(0, catalog::intimidation_campaign());
        drain_stack(&mut g);
        assert_eq!(g.players[1].life, opp - 1, "opponent lost 1");
        assert_eq!(g.players[0].life, me + 1, "you gained 1");
        assert_eq!(g.players[0].hand.len(), hand + 1, "you drew a card");
    }

    /// Eddymurk Crab's ETB taps up to two creatures.
    #[test]
    fn eddymurk_crab_taps_on_etb() {
        let mut g = two_player_game();
        let a = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let b = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        g.decider = Box::new(crabomination::decision::ScriptedDecider::new(
            [crabomination::decision::DecisionAnswer::Cards(vec![a, b])],
        ));
        g.move_card_to_battlefield_for_test(0, catalog::eddymurk_crab());
        drain_stack(&mut g);
        let tapped = [a, b].iter().filter(|&&id| g.battlefield_find(id).map(|c| c.tapped).unwrap_or(false)).count();
        assert_eq!(tapped, 2, "ETB tapped the two chosen creatures");
    }
}

mod recent155 {
    use crabomination::card::Keyword;
    use crabomination::catalog;
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    use crabomination::game::types::Target;
    use crabomination::game::*;
    use crabomination::game::two_player_game;
    use crabomination::mana::Color;

    fn fill_mana(g: &mut GameState) {
        for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
            g.players[0].mana_pool.add(c, 8);
        }
        g.players[0].mana_pool.add_colorless(8);
    }

    /// Auspicious Arrival pumps a creature and makes a Clue.
    #[test]
    fn auspicious_arrival_pumps_and_investigates() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let id = g.add_card_to_hand(0, catalog::auspicious_arrival());
        fill_mana(&mut g);
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast Auspicious Arrival");
        drain_stack(&mut g);
        assert_eq!(g.computed_permanent(bear).map(|c| (c.power, c.toughness)), Some((4, 4)), "+2/+2");
        assert!(g.battlefield.iter().any(|c| c.definition.name == "Clue" && c.controller == 0), "made a Clue");
    }

    /// Benthic Criminologists' ETB sacrifices an artifact to draw.
    #[test]
    fn benthic_criminologists_sac_for_draw() {
        let mut g = two_player_game();
        g.add_token_to_battlefield(0, &crabomination_base::tokens::treasure_token());
        g.add_card_to_library(0, catalog::forest());
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
        let hand = g.players[0].hand.len();
        g.move_card_to_battlefield_for_test(0, catalog::benthic_criminologists());
        drain_stack(&mut g);
        assert_eq!(g.players[0].hand.len(), hand + 1, "sacrificed an artifact and drew");
    }

    /// Agency Coroner sacrifices another creature to draw.
    #[test]
    fn agency_coroner_sac_draw() {
        let mut g = two_player_game();
        let coroner = g.add_card_to_battlefield(0, catalog::agency_coroner());
        g.clear_sickness(coroner);
        let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.add_card_to_library(0, catalog::forest());
        fill_mana(&mut g);
        let hand = g.players[0].hand.len();
        g.perform_action(GameAction::ActivateAbility {
            card_id: coroner, ability_index: 0, target: None, additional_targets: vec![], x_value: None, mode: None,
        }).expect("activate Agency Coroner");
        drain_stack(&mut g);
        assert!(g.battlefield_find(fodder).is_none(), "sacrificed the other creature");
        assert_eq!(g.players[0].hand.len(), hand + 1, "drew a card");
    }

    /// Call a Surprise Witness reanimates a small creature with a flying counter.
    #[test]
    fn call_a_surprise_witness_reanimates_flying() {
        let mut g = two_player_game();
        let dead = g.add_card_to_graveyard(0, catalog::grizzly_bears()); // MV 2
        let id = g.add_card_to_hand(0, catalog::call_a_surprise_witness());
        fill_mana(&mut g);
        g.step = TurnStep::PreCombatMain;
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: Some(Target::Permanent(dead)), additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast Call a Surprise Witness");
        drain_stack(&mut g);
        assert!(g.battlefield_find(dead).is_some(), "reanimated to the battlefield");
        assert!(g.computed_permanent(dead).unwrap().keywords.contains(&Keyword::Flying),
            "the flying counter grants flying");
    }
}

mod recent156 {
    use crabomination::catalog;
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    use crabomination::game::*;

    /// Seedglaive Mentor's Valiant puts a +1/+1 counter on it the first time you
    /// target it each turn.
    #[test]
    fn seedglaive_mentor_valiant_grows() {
        let mut g = two_player_game();
        let m = g.add_card_to_battlefield(0, catalog::seedglaive_mentor());
        g.dispatch_triggers_for_events(&[GameEvent::BecameTarget { target: m, caster: 0 }]);
        drain_stack(&mut g);
        assert_eq!(g.computed_permanent(m).unwrap().power, 4, "Valiant added a +1/+1 counter");
        // Only once per turn.
        g.dispatch_triggers_for_events(&[GameEvent::BecameTarget { target: m, caster: 0 }]);
        drain_stack(&mut g);
        assert_eq!(g.computed_permanent(m).unwrap().power, 4, "fires only once per turn");
    }

    /// Mouse Trapper's Valiant taps a creature an opponent controls.
    #[test]
    fn mouse_trapper_valiant_taps_opponent() {
        let mut g = two_player_game();
        let trapper = g.add_card_to_battlefield(0, catalog::mouse_trapper());
        let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        g.dispatch_triggers_for_events(&[GameEvent::BecameTarget { target: trapper, caster: 0 }]);
        drain_stack(&mut g);
        assert!(g.battlefield_find(foe).unwrap().tapped, "Valiant tapped the opponent's creature");
    }

    /// Flowerfoot Swordmaster's Valiant pumps every Mouse you control +1/+0.
    #[test]
    fn flowerfoot_swordmaster_valiant_pumps_mice() {
        let mut g = two_player_game();
        let master = g.add_card_to_battlefield(0, catalog::flowerfoot_swordmaster());
        let other = g.add_card_to_battlefield(0, catalog::seedglaive_mentor());
        g.dispatch_triggers_for_events(&[GameEvent::BecameTarget { target: master, caster: 0 }]);
        drain_stack(&mut g);
        assert_eq!(g.computed_permanent(master).unwrap().power, 2, "self pumped +1/+0");
        assert_eq!(g.computed_permanent(other).unwrap().power, 4, "other Mouse pumped +1/+0");
    }

    /// Whiskerquill Scribe's Valiant loots — discard a card to draw a card.
    #[test]
    fn whiskerquill_scribe_valiant_loots() {
        let mut g = two_player_game();
        let scribe = g.add_card_to_battlefield(0, catalog::whiskerquill_scribe());
        g.add_card_to_hand(0, catalog::grizzly_bears());
        g.add_card_to_library(0, catalog::forest());
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
        g.dispatch_triggers_for_events(&[GameEvent::BecameTarget { target: scribe, caster: 0 }]);
        drain_stack(&mut g);
        assert_eq!(g.players[0].graveyard.len(), 1, "discarded a card");
        assert!(g.players[0].library.is_empty(), "drew the top of library");
    }
}

mod recent157 {
    use crabomination::card::Keyword;
    use crabomination::catalog;
    use crabomination::game::types::{Attack, AttackTarget, TurnStep};
    use crabomination::game::*;
    use crabomination::mana::Color;

    /// Advance to the active player's declare-attackers step and swing with `id`.
    fn attack_with(g: &mut GameState, id: CardId) {
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

    fn team_power(g: &GameState, seat: usize) -> i32 {
        g.battlefield
            .iter()
            .filter(|c| c.controller == seat && c.definition.is_creature())
            .filter_map(|c| g.computed_permanent(c.id).map(|p| p.power))
            .sum()
    }

    /// Darkstar Augur's upkeep draws the top card and loses life equal to its MV.
    #[test]
    fn darkstar_augur_draws_and_loses_life() {
        let mut g = two_player_game();
        g.players[0].library.clear();
        let cid = g.next_id();
        g.players[0].add_to_library_top(cid, catalog::serra_angel()); // 5 MV
        let augur = g.add_card_to_battlefield(0, catalog::darkstar_augur());
        g.clear_sickness(augur);
        g.active_player_idx = 0;
        g.step = TurnStep::Upkeep;
        g.priority.player_with_priority = 0;
        let life = g.players[0].life;
        let hand = g.players[0].hand.len();
        g.fire_step_triggers(TurnStep::Upkeep);
        drain_stack(&mut g);
        assert_eq!(g.players[0].hand.len(), hand + 1, "drew the top card");
        assert_eq!(life - g.players[0].life, 5, "lost life equal to Serra Angel's MV");
    }

    /// Honored Dreyleader enters with a +1/+1 counter per other Squirrel/Food.
    #[test]
    fn honored_dreyleader_enters_scaled_by_food() {
        let mut g = two_player_game();
        g.add_token_to_battlefield(0, &crabomination_base::tokens::food_token());
        g.add_token_to_battlefield(0, &crabomination_base::tokens::food_token());
        g.move_card_to_battlefield_for_test(0, catalog::honored_dreyleader());
        drain_stack(&mut g);
        let dl = g.battlefield.iter().find(|c| c.definition.name == "Honored Dreyleader").unwrap().id;
        assert_eq!(g.computed_permanent(dl).unwrap().power, 3, "1/1 + two counters from two Food");
    }

    /// Fecund Greenshell's +2/+2 anthem switches on at ten lands.
    #[test]
    fn fecund_greenshell_anthem_at_ten_lands() {
        let mut g = two_player_game();
        let shell = g.add_card_to_battlefield(0, catalog::fecund_greenshell());
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        for _ in 0..9 {
            g.add_card_to_battlefield(0, catalog::forest());
        }
        assert_eq!(g.computed_permanent(bear).unwrap().power, 2, "only nine lands → no anthem");
        g.add_card_to_battlefield(0, catalog::forest());
        assert_eq!(g.computed_permanent(bear).unwrap().power, 4, "ten lands → +2/+2");
        assert_eq!(g.computed_permanent(shell).unwrap().power, 6, "anthem hits itself too");
    }

    /// Hazardroot Herbalist's attack trigger pumps a creature you control.
    #[test]
    fn hazardroot_herbalist_pumps_on_attack() {
        let mut g = two_player_game();
        let herb = g.add_card_to_battlefield(0, catalog::hazardroot_herbalist());
        g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let before = team_power(&g, 0);
        attack_with(&mut g, herb);
        assert_eq!(team_power(&g, 0), before + 1, "+1/+0 to a creature you control");
    }

    /// Rust-Shield Rampager can't be blocked by power 2 or less.
    #[test]
    fn rust_shield_rampager_evasion() {
        let mut g = two_player_game();
        let r = g.add_card_to_battlefield(0, catalog::rust_shield_rampager());
        assert!(g
            .computed_permanent(r)
            .unwrap()
            .keywords
            .contains(&Keyword::CantBeBlockedByPowerAtMost(2)));
    }

    /// Seedpod Squire's attack pumps a non-flying creature you control.
    #[test]
    fn seedpod_squire_pumps_grounded_ally() {
        let mut g = two_player_game();
        let squire = g.add_card_to_battlefield(0, catalog::seedpod_squire());
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        attack_with(&mut g, squire);
        assert_eq!(g.computed_permanent(bear).unwrap().power, 3, "grounded bear got +1/+1");
    }

    /// Steampath Charger's death deals 1 to a player.
    #[test]
    fn steampath_charger_death_pings() {
        let mut g = two_player_game();
        let charger = g.add_card_to_battlefield(0, catalog::steampath_charger());
        let life = g.players[1].life;
        let mut evs = g.remove_to_graveyard_with_triggers(charger);
        evs.push(GameEvent::CreatureDied { card_id: charger });
        g.dispatch_triggers_for_events(&evs);
        drain_stack(&mut g);
        assert_eq!(g.players[1].life, life - 1, "death dealt 1 damage");
    }

    /// Treeguard Duo's ETB grants +X/+X where X is creatures you control.
    #[test]
    fn treeguard_duo_pumps_by_creature_count() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let before = team_power(&g, 0);
        g.move_card_to_battlefield_for_test(0, catalog::treeguard_duo());
        drain_stack(&mut g);
        // Two creatures you control at resolution → +2/+2 to one of them.
        assert_eq!(team_power(&g, 0), before + 3 /* Duo's own 3 power */ + 2);
    }

    /// Junkblade Bruiser grows when you expend 4.
    #[test]
    fn junkblade_bruiser_expend_pumps() {
        let mut g = two_player_game();
        let bruiser = g.add_card_to_battlefield(0, catalog::junkblade_bruiser());
        let moose = g.add_card_to_hand(0, catalog::galewind_moose()); // {4}{G}{G}
        g.players[0].mana_pool.add(Color::Green, 2);
        g.players[0].mana_pool.add_colorless(4);
        g.perform_action(GameAction::CastSpell {
            card_id: moose, target: None, additional_targets: vec![], mode: None, x_value: None,
        })
        .expect("cast a 6-mana spell (crosses expend 4)");
        drain_stack(&mut g);
        assert_eq!(g.computed_permanent(bruiser).unwrap().power, 6, "expend 4 → +2/+1");
    }

    /// Waterspout Warden gains flying when another creature entered this turn.
    #[test]
    fn waterspout_warden_conditional_flying() {
        let mut g = two_player_game();
        let warden = g.add_card_to_battlefield(0, catalog::waterspout_warden());
        let ally = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.players[0].creatures_entered_this_turn.push(ally);
        attack_with(&mut g, warden);
        assert!(
            g.computed_permanent(warden).unwrap().keywords.contains(&Keyword::Flying),
            "gained flying after another creature entered this turn"
        );
    }
}

mod recent158 {
    use crabomination::card::Keyword;
    use crabomination::catalog;
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    use crabomination::game::types::{Target, TurnStep};
    use crabomination::game::*;
    use crabomination::mana::Color;

    fn fill_mana(g: &mut GameState) {
        for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
            g.players[0].mana_pool.add(c, 8);
        }
        g.players[0].mana_pool.add_colorless(8);
    }

    /// Starlit Soothsayer surveils at end step only if you gained or lost life.
    #[test]
    fn starlit_soothsayer_surveils_after_life_change() {
        let mut g = two_player_game();
        let sooth = g.add_card_to_battlefield(0, catalog::starlit_soothsayer());
        g.clear_sickness(sooth);
        g.players[0].library.clear();
        let top = g.next_id();
        g.players[0].add_to_library_top(top, catalog::island());
        g.players[0].life_gained_this_turn = 2;
        g.active_player_idx = 0;
        g.step = TurnStep::End;
        g.priority.player_with_priority = 0;
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::ScryOrder {
            kept_top: vec![],
            bottom: vec![top],
        }]));
        g.fire_step_triggers(TurnStep::End);
        drain_stack(&mut g);
        assert!(g.players[0].graveyard.iter().any(|c| c.id == top), "surveiled the top card to gy");
    }

    /// Omenport Vigilante gains double strike only after you commit a crime.
    #[test]
    fn omenport_vigilante_double_strike_on_crime() {
        let mut g = two_player_game();
        let v = g.add_card_to_battlefield(0, catalog::omenport_vigilante());
        assert!(!g.computed_permanent(v).unwrap().keywords.contains(&Keyword::DoubleStrike));
        g.players[0].committed_crime_this_turn = true;
        assert!(g.computed_permanent(v).unwrap().keywords.contains(&Keyword::DoubleStrike), "crime → double strike");
    }

    /// Essence Channeler flies after you lose life, and grows on lifegain.
    #[test]
    fn essence_channeler_lost_life_flying_and_grows() {
        let mut g = two_player_game();
        let ec = g.add_card_to_battlefield(0, catalog::essence_channeler());
        assert!(!g.computed_permanent(ec).unwrap().keywords.contains(&Keyword::Flying));
        g.players[0].lost_life_this_turn = true;
        let c = g.computed_permanent(ec).unwrap();
        assert!(c.keywords.contains(&Keyword::Flying) && c.keywords.contains(&Keyword::Vigilance), "lost life → flying + vigilance");
        // Gaining life adds a +1/+1 counter.
        g.dispatch_triggers_for_events(&[GameEvent::LifeGained { player: 0, amount: 3 }]);
        drain_stack(&mut g);
        assert_eq!(g.computed_permanent(ec).unwrap().power, 3, "lifegain grew it");
    }

    /// Cactarantula draws when an opponent targets it.
    #[test]
    fn cactarantula_draws_on_opponent_target() {
        let mut g = two_player_game();
        let cact = g.add_card_to_battlefield(0, catalog::cactarantula());
        g.add_card_to_library(0, catalog::forest());
        let hand = g.players[0].hand.len();
        g.dispatch_triggers_for_events(&[GameEvent::BecameTarget { target: cact, caster: 1 }]);
        drain_stack(&mut g);
        assert_eq!(g.players[0].hand.len(), hand + 1, "drew off an opponent's target");
    }

    /// Inventive Wingsmith wings itself at end step if you cast no spells.
    #[test]
    fn inventive_wingsmith_gets_flying_counter() {
        let mut g = two_player_game();
        let smith = g.add_card_to_battlefield(0, catalog::inventive_wingsmith());
        g.clear_sickness(smith);
        g.active_player_idx = 0;
        g.step = TurnStep::End;
        g.priority.player_with_priority = 0;
        g.fire_step_triggers(TurnStep::End);
        drain_stack(&mut g);
        assert!(g.computed_permanent(smith).unwrap().keywords.contains(&Keyword::Flying), "gained a flying counter");
    }

    /// Mourner's Surprise returns a creature card and mints a Mercenary.
    #[test]
    fn mourners_surprise_reanimates_to_hand_and_makes_token() {
        let mut g = two_player_game();
        let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
        let id = g.add_card_to_hand(0, catalog::mourners_surprise());
        fill_mana(&mut g);
        g.step = TurnStep::PreCombatMain;
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast Mourner's Surprise");
        drain_stack(&mut g);
        assert!(g.players[0].hand.iter().any(|c| c.id == bear), "returned the creature card to hand");
        assert!(g.battlefield.iter().any(|c| c.definition.name == "Mercenary" && c.controller == 0), "made a Mercenary");
    }
}

mod recent159 {
    use crabomination::card::Keyword;
    use crabomination::catalog;
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    use crabomination::game::types::{Target, TurnStep};
    use crabomination::game::*;
    use crabomination::mana::Color;

    fn fill_mana(g: &mut GameState) {
        for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
            g.players[0].mana_pool.add(c, 8);
        }
        g.players[0].mana_pool.add_colorless(8);
    }

    /// Fanatical Strength pumps +3/+3 and grants trample.
    #[test]
    fn fanatical_strength_pumps_and_tramples() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let id = g.add_card_to_hand(0, catalog::fanatical_strength());
        fill_mana(&mut g);
        g.step = TurnStep::PreCombatMain;
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast Fanatical Strength");
        drain_stack(&mut g);
        let c = g.computed_permanent(bear).unwrap();
        assert_eq!((c.power, c.toughness), (5, 5), "+3/+3");
        assert!(c.keywords.contains(&Keyword::Trample), "gained trample");
    }

    /// Festerleech's activated pump only fires once each turn.
    #[test]
    fn festerleech_pump_once_per_turn() {
        let mut g = two_player_game();
        let fl = g.add_card_to_battlefield(0, catalog::festerleech());
        g.clear_sickness(fl);
        fill_mana(&mut g);
        for _ in 0..2 {
            let _ = g.perform_action(GameAction::ActivateAbility {
                card_id: fl, ability_index: 0, target: None, additional_targets: vec![], x_value: None, mode: None,
            });
            drain_stack(&mut g);
        }
        assert_eq!(g.computed_permanent(fl).unwrap().power, 3, "one activation only: 1 + 2");
    }

    /// Cornered Crook sacrifices an artifact to deal 3 damage.
    #[test]
    fn cornered_crook_sac_for_damage() {
        let mut g = two_player_game();
        g.add_token_to_battlefield(0, &crabomination_base::tokens::treasure_token());
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
        let life = g.players[1].life;
        g.move_card_to_battlefield_for_test(0, catalog::cornered_crook());
        drain_stack(&mut g);
        assert_eq!(g.players[1].life, life - 3, "sacrificed the Treasure and dealt 3");
    }

    /// Crime Novelist grows and ramps when you sacrifice an artifact.
    #[test]
    fn crime_novelist_grows_on_artifact_sacrifice() {
        let mut g = two_player_game();
        let cn = g.add_card_to_battlefield(0, catalog::crime_novelist());
        let treasure = g.add_token_to_battlefield(0, &crabomination_base::tokens::treasure_token());
        let mut evs = g.remove_to_graveyard_with_triggers(treasure);
        evs.push(GameEvent::PermanentSacrificed { card_id: treasure, who: 0 });
        g.dispatch_triggers_for_events(&evs);
        drain_stack(&mut g);
        assert_eq!(g.computed_permanent(cn).unwrap().power, 2, "got a +1/+1 counter");
    }

    /// Absolving Lammasu clears suspicion on entry and suspects on death.
    #[test]
    fn absolving_lammasu_clears_then_suspects() {
        let mut g = two_player_game();
        // A friendly creature is suspected; ETB clears it.
        let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.battlefield_find_mut(mine).unwrap().suspected = true;
        let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let lam = g.add_card_to_battlefield(0, catalog::absolving_lammasu());
        g.fire_self_etb_triggers(lam, 0);
        drain_stack(&mut g);
        assert!(!g.battlefield_find(mine).unwrap().suspected, "ETB cleared suspicion");
        // Death suspects an opponent's creature.
        let life = g.players[0].life;
        let mut evs = g.remove_to_graveyard_with_triggers(lam);
        evs.push(GameEvent::CreatureDied { card_id: lam });
        g.dispatch_triggers_for_events(&evs);
        drain_stack(&mut g);
        assert_eq!(g.players[0].life, life + 3, "gained 3 life");
        assert!(g.battlefield_find(foe).unwrap().suspected, "suspected the opponent's creature");
    }
}

mod recent160 {
    use crabomination::catalog;
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    use crabomination::game::types::TurnStep;
    use crabomination::game::*;
    use crabomination::mana::Color;

    fn fill_mana(g: &mut GameState) {
        for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
            g.players[0].mana_pool.add(c, 8);
        }
        g.players[0].mana_pool.add_colorless(8);
    }

    /// Erudite Wizard grows when you draw your second card in a turn.
    #[test]
    fn erudite_wizard_grows_on_second_draw() {
        let mut g = two_player_game();
        let wiz = g.add_card_to_battlefield(0, catalog::erudite_wizard());
        g.add_card_to_library(0, catalog::island());
        g.add_card_to_library(0, catalog::island());
        let mut evs = vec![];
        g.draw_one(0, &mut evs);
        g.draw_one(0, &mut evs);
        g.dispatch_triggers_for_events(&evs);
        drain_stack(&mut g);
        assert_eq!(g.computed_permanent(wiz).unwrap().power, 3, "second draw → +1/+1");
    }

    /// Gorehorn Raider's Raid pings when you attacked this turn.
    #[test]
    fn gorehorn_raider_raid_pings() {
        let mut g = two_player_game();
        g.players[0].attacked_this_turn = true;
        let life = g.players[1].life;
        let raider = g.add_card_to_battlefield(0, catalog::gorehorn_raider());
        g.fire_self_etb_triggers(raider, 0);
        drain_stack(&mut g);
        assert_eq!(g.players[1].life, life - 2, "Raid dealt 2");
    }

    /// Gutless Plunderer digs three on Raid.
    #[test]
    fn gutless_plunderer_raid_digs() {
        let mut g = two_player_game();
        g.players[0].attacked_this_turn = true;
        g.players[0].library.clear();
        for _ in 0..3 {
            g.add_card_to_library(0, catalog::island());
        }
        let gy = g.players[0].graveyard.len();
        let plunderer = g.add_card_to_battlefield(0, catalog::gutless_plunderer());
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::ScryOrder {
            kept_top: vec![],
            bottom: vec![],
        }]));
        g.fire_self_etb_triggers(plunderer, 0);
        drain_stack(&mut g);
        assert_eq!(g.players[0].graveyard.len(), gy + 2, "kept one, milled the other two");
    }

    /// Hinterland Sanctifier gains life when another creature you control enters.
    #[test]
    fn hinterland_sanctifier_gains_life() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::hinterland_sanctifier());
        let ally = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let life = g.players[0].life;
        g.dispatch_triggers_for_events(&[GameEvent::PermanentEntered { card_id: ally }]);
        drain_stack(&mut g);
        assert_eq!(g.players[0].life, life + 1, "gained 1 life");
    }

    /// Hungry Ghoul grows by sacrificing another creature.
    #[test]
    fn hungry_ghoul_sac_grows() {
        let mut g = two_player_game();
        let ghoul = g.add_card_to_battlefield(0, catalog::hungry_ghoul());
        g.clear_sickness(ghoul);
        let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        fill_mana(&mut g);
        g.perform_action(GameAction::ActivateAbility {
            card_id: ghoul, ability_index: 0, target: None, additional_targets: vec![], x_value: None, mode: None,
        }).expect("activate Hungry Ghoul");
        drain_stack(&mut g);
        assert!(g.battlefield_find(fodder).is_none(), "sacrificed the fodder");
        assert_eq!(g.computed_permanent(ghoul).unwrap().power, 3, "grew +1/+1");
    }

    /// Icewind Elemental loots on entry.
    #[test]
    fn icewind_elemental_loots() {
        let mut g = two_player_game();
        g.add_card_to_library(0, catalog::island());
        let forest = g.add_card_to_hand(0, catalog::forest());
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Discard(vec![forest])]));
        let hand = g.players[0].hand.len();
        g.move_card_to_battlefield_for_test(0, catalog::icewind_elemental());
        drain_stack(&mut g);
        // Drew one and discarded one → net hand unchanged, graveyard grew.
        assert_eq!(g.players[0].hand.len(), hand, "drew one, discarded one");
        assert!(g.players[0].graveyard.iter().any(|c| c.id == forest), "discarded the forest");
    }

    /// Infestation Sage leaves a flying Insect when it dies.
    #[test]
    fn infestation_sage_dies_to_insect() {
        let mut g = two_player_game();
        let sage = g.add_card_to_battlefield(0, catalog::infestation_sage());
        let mut evs = g.remove_to_graveyard_with_triggers(sage);
        evs.push(GameEvent::CreatureDied { card_id: sage });
        g.dispatch_triggers_for_events(&evs);
        drain_stack(&mut g);
        assert!(g.battlefield.iter().any(|c| c.definition.name == "Insect" && c.controller == 0), "made an Insect");
    }

    /// Prideful Parent brings a Cat friend.
    #[test]
    fn prideful_parent_makes_a_cat() {
        let mut g = two_player_game();
        g.move_card_to_battlefield_for_test(0, catalog::prideful_parent());
        drain_stack(&mut g);
        assert!(g.battlefield.iter().any(|c| c.definition.name == "Cat" && c.controller == 0), "made a Cat");
    }

    /// Firespitter Whelp pings each opponent on a noncreature cast.
    #[test]
    fn firespitter_whelp_pings_on_noncreature() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::firespitter_whelp());
        g.add_card_to_library(0, catalog::island());
        g.add_card_to_library(0, catalog::island());
        let id = g.add_card_to_hand(0, catalog::divination());
        fill_mana(&mut g);
        g.step = TurnStep::PreCombatMain;
        let life = g.players[1].life;
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast Divination");
        drain_stack(&mut g);
        assert_eq!(g.players[1].life, life - 1, "noncreature cast pinged the opponent");
    }

    /// Guarded Heir brings two Knights.
    #[test]
    fn guarded_heir_makes_two_knights() {
        let mut g = two_player_game();
        g.move_card_to_battlefield_for_test(0, catalog::guarded_heir());
        drain_stack(&mut g);
        let knights = g.battlefield.iter().filter(|c| c.definition.name == "Knight" && c.controller == 0).count();
        assert_eq!(knights, 2, "made two Knights");
    }
}

mod recent161 {
    use crabomination::catalog;
    use crabomination::game::types::{Attack, AttackTarget, Target, TurnStep};
    use crabomination::game::*;
    use crabomination::mana::Color;

    fn fill_mana(g: &mut GameState) {
        for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
            g.players[0].mana_pool.add(c, 8);
        }
        g.players[0].mana_pool.add_colorless(8);
    }

    /// Incinerating Blast deals 6 to a creature.
    #[test]
    fn incinerating_blast_burns() {
        let mut g = two_player_game();
        let foe = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4
        let id = g.add_card_to_hand(0, catalog::incinerating_blast());
        fill_mana(&mut g);
        g.step = TurnStep::PreCombatMain;
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: Some(Target::Permanent(foe)), additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast Incinerating Blast");
        drain_stack(&mut g);
        assert!(g.battlefield_find(foe).is_none(), "6 damage killed the 4/4");
    }

    /// Needletooth Pack's Morbid grows a creature after a death.
    #[test]
    fn needletooth_pack_morbid_grows() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::needletooth_pack());
        let ally = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        // A creature died this turn.
        let chump = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.remove_to_graveyard_with_triggers(chump);
        g.active_player_idx = 0;
        g.step = TurnStep::End;
        g.priority.player_with_priority = 0;
        g.fire_step_triggers(TurnStep::End);
        drain_stack(&mut g);
        // Two +1/+1 counters landed on a creature you control.
        let bumped = g.computed_permanent(ally).unwrap().power == 4
            || g.battlefield.iter().any(|c| c.definition.name == "Needletooth Pack" && c.controller == 0 && g.computed_permanent(c.id).unwrap().power == 6);
        assert!(bumped, "Morbid added two +1/+1 counters");
    }

    /// Grappling Kraken taps and stuns on landfall.
    #[test]
    fn grappling_kraken_landfall_stuns() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::grappling_kraken());
        let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let land = g.add_card_to_hand(0, catalog::forest());
        g.active_player_idx = 0;
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::PlayLand(land)).expect("play a land");
        drain_stack(&mut g);
        let c = g.battlefield_find(foe).unwrap();
        assert!(c.tapped, "landfall tapped the opponent's creature");
        assert!(c.counters.get(&crabomination::card::CounterType::Stun).copied().unwrap_or(0) >= 1, "stun counter placed");
    }

    /// Joust Through hits an attacking creature. (Player 0 swings, then burns their
    /// own attacker during combat — exercises the attacking/blocking target filter.)
    #[test]
    fn joust_through_hits_attacker() {
        let mut g = two_player_game();
        let attacker = g.add_card_to_battlefield(0, catalog::serra_angel());
        g.clear_sickness(attacker);
        let id = g.add_card_to_hand(0, catalog::joust_through());
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
        g.players[0].mana_pool.add(Color::White, 1);
        let life = g.players[0].life;
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: Some(Target::Permanent(attacker)), additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast Joust Through");
        drain_stack(&mut g);
        assert_eq!(g.battlefield_find(attacker).map(|c| c.damage), Some(3), "3 damage to the attacker");
        assert_eq!(g.players[0].life, life + 1, "gained 1 life");
    }

    /// Quakestrider Ceratops is a 12/8 vanilla.
    #[test]
    fn quakestrider_ceratops_is_a_giant() {
        let mut g = two_player_game();
        let c = g.add_card_to_battlefield(0, catalog::quakestrider_ceratops());
        let p = g.computed_permanent(c).unwrap();
        assert_eq!((p.power, p.toughness), (12, 8));
    }
}

mod recent162 {
    use crabomination::catalog;
    use crabomination::card::{CounterType, Keyword, WardCost};
    use crabomination::game::types::{Attack, AttackTarget, Target, TurnStep};
    use crabomination::game::*;
    use crabomination::mana::Color;

    fn fill_mana(g: &mut GameState) {
        for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
            g.players[0].mana_pool.add(c, 8);
        }
        g.players[0].mana_pool.add_colorless(8);
    }

    /// Felling Blow grows your creature, then that creature swings for its power.
    #[test]
    fn felling_blow_pumps_and_fights() {
        let mut g = two_player_game();
        let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
        let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2 → 3/3? no, 2/2
        let id = g.add_card_to_hand(0, catalog::felling_blow());
        fill_mana(&mut g);
        g.step = TurnStep::PreCombatMain;
        g.perform_action(GameAction::CastSpell {
            card_id: id,
            target: Some(Target::Permanent(mine)),
            additional_targets: vec![Target::Permanent(foe)],
            mode: None,
            x_value: None,
        })
        .expect("cast Felling Blow");
        drain_stack(&mut g);
        // Counter made it a 3/3; 3 damage kills the opposing 2/2.
        assert_eq!(g.computed_permanent(mine).map(|c| c.power), Some(3), "+1/+1 counter");
        assert!(g.battlefield_find(foe).is_none(), "took lethal damage equal to power");
    }

    /// Inspiration from Beyond mills and returns an instant/sorcery.
    #[test]
    fn inspiration_from_beyond_returns_spell() {
        let mut g = two_player_game();
        for _ in 0..3 {
            g.add_card_to_library(0, catalog::island());
        }
        let bolt = g.add_card_to_graveyard(0, catalog::lightning_bolt());
        let id = g.add_card_to_hand(0, catalog::inspiration_from_beyond());
        fill_mana(&mut g);
        g.step = TurnStep::PreCombatMain;
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
        })
        .expect("cast Inspiration from Beyond");
        drain_stack(&mut g);
        assert!(g.players[0].hand.iter().any(|c| c.id == bolt), "returned the instant to hand");
    }

    /// Sower of Chaos grants can't-block to a target creature.
    #[test]
    fn sower_of_chaos_grants_cant_block() {
        let mut g = two_player_game();
        let sower = g.add_card_to_battlefield(0, catalog::sower_of_chaos());
        let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        g.players[0].mana_pool.add(Color::Red, 3);
        g.perform_action(GameAction::ActivateAbility {
            card_id: sower, ability_index: 0,
            target: Some(Target::Permanent(foe)), additional_targets: vec![], x_value: None, mode: None,
        })
        .expect("activate can't-block");
        drain_stack(&mut g);
        assert!(g.computed_permanent(foe).unwrap().keywords.contains(&Keyword::CantBlock));
    }

    /// Searslicer Goblin's Raid mints a Goblin at end step after you attacked.
    #[test]
    fn searslicer_goblin_raid_token() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::searslicer_goblin());
        g.players[0].attacked_this_turn = true;
        g.active_player_idx = 0;
        g.step = TurnStep::End;
        g.priority.player_with_priority = 0;
        g.fire_step_triggers(TurnStep::End);
        drain_stack(&mut g);
        assert!(
            g.battlefield.iter().any(|c| c.definition.name == "Goblin" && c.controller == 0),
            "Raid created a Goblin token"
        );
    }

    /// Sire of Seven Deaths is a 7/7 with the full keyword pile and Ward—pay 7 life.
    #[test]
    fn sire_of_seven_deaths_keywords() {
        let mut g = two_player_game();
        let c = g.add_card_to_battlefield(0, catalog::sire_of_seven_deaths());
        let cp = g.computed_permanent(c).unwrap();
        assert_eq!((cp.power, cp.toughness), (7, 7));
        for k in [Keyword::FirstStrike, Keyword::Vigilance, Keyword::Menace, Keyword::Trample, Keyword::Reach, Keyword::Lifelink] {
            assert!(cp.keywords.contains(&k), "missing {k:?}");
        }
        assert!(cp.keywords.contains(&Keyword::Ward(WardCost::Life(7))), "Ward—pay 7 life");
    }

    /// Preposterous Proportions gives your team +10/+10 and vigilance.
    #[test]
    fn preposterous_proportions_pumps_team() {
        let mut g = two_player_game();
        let a = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let b = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let id = g.add_card_to_hand(0, catalog::preposterous_proportions());
        fill_mana(&mut g);
        g.step = TurnStep::PreCombatMain;
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
        })
        .expect("cast Preposterous Proportions");
        drain_stack(&mut g);
        for id in [a, b] {
            let cp = g.computed_permanent(id).unwrap();
            assert_eq!((cp.power, cp.toughness), (12, 12), "+10/+10");
            assert!(cp.keywords.contains(&Keyword::Vigilance), "gained vigilance");
        }
    }

    /// Slumbering Cerberus won't untap normally, but Morbid untaps it after a death.
    #[test]
    fn slumbering_cerberus_morbid_untaps() {
        let mut g = two_player_game();
        let dog = g.add_card_to_battlefield(0, catalog::slumbering_cerberus());
        g.battlefield_find_mut(dog).unwrap().tapped = true;
        // A creature died this turn.
        let chump = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.remove_to_graveyard_with_triggers(chump);
        g.active_player_idx = 0;
        g.step = TurnStep::End;
        g.priority.player_with_priority = 0;
        g.fire_step_triggers(TurnStep::End);
        drain_stack(&mut g);
        assert!(!g.battlefield_find(dog).unwrap().tapped, "Morbid untapped the Cerberus");
    }

    /// Squad Rallier digs a small creature into hand.
    #[test]
    fn squad_rallier_digs_creature() {
        let mut g = two_player_game();
        let rallier = g.add_card_to_battlefield(0, catalog::squad_rallier());
        g.add_card_to_library(0, catalog::grizzly_bears()); // power 2 → eligible
        for _ in 0..3 {
            g.add_card_to_library(0, catalog::island());
        }
        g.players[0].mana_pool.add(Color::White, 3);
        let hand = g.players[0].hand.len();
        g.perform_action(GameAction::ActivateAbility {
            card_id: rallier, ability_index: 0, target: None, additional_targets: vec![], x_value: None, mode: None,
        })
        .expect("activate dig");
        drain_stack(&mut g);
        assert!(g.players[0].hand.len() > hand, "dug a creature into hand");
    }

    /// Sphinx of Forgotten Lore grants flashback to a graveyard spell on attack.
    #[test]
    fn sphinx_grants_flashback_on_attack() {
        let mut g = two_player_game();
        let sphinx = g.add_card_to_battlefield(0, catalog::sphinx_of_forgotten_lore());
        g.clear_sickness(sphinx);
        let bolt = g.add_card_to_graveyard(0, catalog::lightning_bolt());
        g.step = TurnStep::DeclareAttackers;
        g.active_player_idx = 0;
        g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker: sphinx, target: AttackTarget::Player(1),
        }]))
        .expect("attack");
        drain_stack(&mut g);
        assert!(
            g.players[0].graveyard.iter().find(|c| c.id == bolt).unwrap().granted_flashback_eot.is_some(),
            "the graveyard spell gained flashback"
        );
    }

    /// Claws Out pumps your whole team.
    #[test]
    fn claws_out_pumps_team() {
        let mut g = two_player_game();
        let a = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let id = g.add_card_to_hand(0, catalog::claws_out());
        fill_mana(&mut g);
        g.step = TurnStep::PreCombatMain;
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
        })
        .expect("cast Claws Out");
        drain_stack(&mut g);
        let cp = g.computed_permanent(a).unwrap();
        assert_eq!((cp.power, cp.toughness), (4, 4), "+2/+2");
    }

    /// Skyknight Squire gains flying once it reaches three +1/+1 counters.
    #[test]
    fn skyknight_squire_flies_at_three() {
        let mut g = two_player_game();
        let squire = g.add_card_to_battlefield(0, catalog::skyknight_squire());
        g.step = TurnStep::PreCombatMain;
        for _ in 0..3 {
            let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
            g.players[0].mana_pool.add(Color::Green, 1);
            g.players[0].mana_pool.add_colorless(1);
            g.perform_action(GameAction::CastSpell {
                card_id: bear, target: None, additional_targets: vec![], mode: None, x_value: None,
            })
            .expect("cast a creature to trigger the Squire");
            drain_stack(&mut g);
        }
        assert_eq!(g.battlefield_find(squire).unwrap().counter_count(CounterType::PlusOnePlusOne), 3);
        assert!(g.computed_permanent(squire).unwrap().keywords.contains(&Keyword::Flying), "3 counters → flying");
    }

    /// Luminous Rebuke destroys a creature, and is cheaper against a tapped one.
    #[test]
    fn luminous_rebuke_destroys_tapped_cheaply() {
        let mut g = two_player_game();
        let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        g.battlefield_find_mut(foe).unwrap().tapped = true;
        let id = g.add_card_to_hand(0, catalog::luminous_rebuke());
        // {4}{W} - {3} = {1}{W} against a tapped creature.
        g.players[0].mana_pool.add(Color::White, 1);
        g.players[0].mana_pool.add_colorless(1);
        g.step = TurnStep::PreCombatMain;
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: Some(Target::Permanent(foe)), additional_targets: vec![], mode: None, x_value: None,
        })
        .expect("cast Luminous Rebuke for its reduced cost");
        drain_stack(&mut g);
        assert!(g.battlefield_find(foe).is_none(), "destroyed the tapped creature");
    }
}
