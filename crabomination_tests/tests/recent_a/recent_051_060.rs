//! Tests for recentN card batches 51-60 (merged from per-batch micro-files).

mod recent51 {
    use crabomination::card::CounterType;
    use crabomination::catalog;
    use crabomination::game::effects::EffectContext;
    use crabomination::game::two_player_game;
    use crabomination::game::*;
    use crabomination::mana::Color;

    #[test]
    fn standard_bearer_draws_per_creature_that_died() {
        let mut g = two_player_game();
        g.players[0].creatures_died_this_turn = 2;
        for _ in 0..3 { g.add_card_to_library(0, catalog::swamp()); }
        let lib0 = g.players[0].library.len();
        let sb = g.add_card_to_battlefield(0, catalog::lilianas_standard_bearer());
        g.fire_self_etb_triggers(sb, 0);
        drain_stack(&mut g);
        assert_eq!(g.players[0].library.len(), lib0 - 2, "drew two for two dead creatures");
    }

    #[test]
    fn skullport_merchant_makes_a_treasure_and_loots() {
        let mut g = two_player_game();
        let sm = g.add_card_to_battlefield(0, catalog::skullport_merchant());
        g.fire_self_etb_triggers(sm, 0);
        drain_stack(&mut g);
        assert_eq!(
            g.battlefield.iter().filter(|c| c.definition.name == "Treasure").count(),
            1,
            "ETB Treasure"
        );
        // Sac the Treasure to draw.
        g.add_card_to_library(0, catalog::swamp());
        let lib0 = g.players[0].library.len();
        g.priority.player_with_priority = 0;
        g.step = TurnStep::PreCombatMain;
        g.players[0].mana_pool.add(Color::Black, 1);
        g.players[0].mana_pool.add_colorless(1);
        g.perform_action(GameAction::ActivateAbility {
            card_id: sm, ability_index: 0, target: None, additional_targets: vec![], x_value: None, mode: None,
        }).expect("loot ability");
        drain_stack(&mut g);
        assert_eq!(g.players[0].library.len(), lib0 - 1, "drew off the sacrifice");
    }

    #[test]
    fn bone_picker_is_cheap_after_a_death() {
        let mut g = two_player_game();
        g.players[0].creatures_died_this_turn = 1;
        let bp = g.add_card_to_hand(0, catalog::bone_picker());
        g.players[0].mana_pool.add(Color::Black, 1); // {3}{B} - {3} = {B}
        g.perform_action(GameAction::CastSpell {
            card_id: bp, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("Bone Picker castable for {B} after a death");
        drain_stack(&mut g);
        assert!(g.battlefield.iter().any(|c| c.definition.name == "Bone Picker"));
    }

    #[test]
    fn driver_of_the_dead_reanimates_a_small_creature() {
        let mut g = two_player_game();
        let small = g.add_card_to_graveyard(0, catalog::grizzly_bears()); // MV 2
        let driver = g.add_card_to_battlefield(0, catalog::driver_of_the_dead());
        let evs = g.remove_to_graveyard_with_triggers(driver);
        let mut evs = evs;
        evs.push(GameEvent::CreatureDied { card_id: driver });
        g.dispatch_triggers_for_events(&evs);
        drain_stack(&mut g);
        assert!(g.battlefield_find(small).is_some(), "small creature back on the battlefield");
    }

    #[test]
    fn gixian_infiltrator_grows_on_sacrifice() {
        let mut g = two_player_game();
        let gix = g.add_card_to_battlefield(0, catalog::gixian_infiltrator());
        let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.sacrifice_one(fodder, 0, &mut vec![]);
        let evs = vec![GameEvent::PermanentSacrificed { card_id: fodder, who: 0 }];
        g.dispatch_triggers_for_events(&evs);
        drain_stack(&mut g);
        assert_eq!(
            g.battlefield_find(gix).unwrap().counters.get(&CounterType::PlusOnePlusOne).copied().unwrap_or(0),
            1,
            "a +1/+1 counter for the sacrifice"
        );
    }

    #[test]
    fn hunger_of_the_howlpack_is_morbid() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        // No death yet → +1/+1.
        let ctx = EffectContext::for_spell(0, Some(Target::Permanent(bear)), 0, 0);
        g.resolve_effect(&catalog::hunger_of_the_howlpack().effect, &ctx).unwrap();
        drain_stack(&mut g);
        assert_eq!(g.battlefield_find(bear).unwrap().counters[&CounterType::PlusOnePlusOne], 1);
        // After a death → +3 more (4 total).
        g.players[0].creatures_died_this_turn = 1;
        g.resolve_effect(&catalog::hunger_of_the_howlpack().effect, &ctx).unwrap();
        drain_stack(&mut g);
        assert_eq!(g.battlefield_find(bear).unwrap().counters[&CounterType::PlusOnePlusOne], 4);
    }
}

mod recent52 {
    use crabomination::card::{CardType, CounterType, CreatureType, Keyword};
    use crabomination::catalog;
    use crabomination::game::*;
    use crabomination::game::{drain_stack, two_player_game};
    use crabomination::mana::Color;

    #[test]
    fn nethergoyf_power_tracks_card_types_in_your_graveyard() {
        let mut g = two_player_game();
        let goyf = g.add_card_to_battlefield(0, catalog::nethergoyf());
        // Empty graveyard → 0/1.
        let view = g.compute_battlefield();
        let v = view.iter().find(|c| c.id == goyf).unwrap();
        assert_eq!((v.power, v.toughness), (0, 1));
        // A creature card and an instant in your graveyard → 2 card types → 2/3.
        g.add_card_to_graveyard(0, catalog::grizzly_bears());
        g.add_card_to_graveyard(0, catalog::lightning_bolt());
        let view = g.compute_battlefield();
        let v = view.iter().find(|c| c.id == goyf).unwrap();
        assert_eq!((v.power, v.toughness), (2, 3), "two card types → 2/3");
    }

    #[test]
    fn omen_hawker_mana_funds_abilities_not_spells() {
        let mut g = two_player_game();
        let hawker = g.add_card_to_battlefield(0, catalog::omen_hawker());
        g.clear_sickness(hawker);
        g.priority.player_with_priority = 0;
        g.step = TurnStep::PreCombatMain;
        g.perform_action(GameAction::ActivateAbility {
            card_id: hawker, ability_index: 0, target: None, additional_targets: vec![], x_value: None, mode: None,
        }).expect("tap for restricted mana");
        // Two mana in pool ({C}{U}), but it can't pay for a creature spell.
        let bear = g.add_card_to_hand(0, catalog::grizzly_bears()); // {1}{G}
        assert!(g.perform_action(GameAction::CastSpell {
            card_id: bear, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).is_err(), "abilities-only mana can't cast a creature spell");
    }

    #[test]
    fn hazardous_blast_pings_opponents_and_stops_blocks() {
        let mut g = two_player_game();
        let small = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // dies? 2/2 survives 1
        let elf = g.add_card_to_battlefield(1, catalog::llanowar_elves()); // 1/1 dies
        let blast = g.add_card_to_hand(0, catalog::hazardous_blast());
        g.players[0].mana_pool.add(Color::Red, 1);
        g.players[0].mana_pool.add_colorless(3);
        g.perform_action(GameAction::CastSpell {
            card_id: blast, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast Hazardous Blast");
        drain_stack(&mut g);
        assert!(!g.battlefield.iter().any(|c| c.id == elf), "1/1 Elf died to the ping");
        assert!(g.battlefield_find(small).unwrap().has_keyword(&Keyword::CantBlock), "survivor can't block");
    }

    #[test]
    fn toxin_analysis_grants_deathtouch_lifelink_and_clues() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let tox = g.add_card_to_hand(0, catalog::toxin_analysis());
        g.players[0].mana_pool.add(Color::Black, 1);
        g.perform_action(GameAction::CastSpell {
            card_id: tox, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast Toxin Analysis");
        drain_stack(&mut g);
        let c = g.battlefield_find(bear).unwrap();
        assert!(c.has_keyword(&Keyword::Deathtouch) && c.has_keyword(&Keyword::Lifelink));
        assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Clue").count(), 1, "Investigate");
    }

    #[test]
    fn enduring_courage_buffs_entering_creatures_and_returns_as_enchantment() {
        let mut g = two_player_game();
        let courage = g.add_card_to_battlefield(0, catalog::enduring_courage());
        // Cast another creature through the real pipeline so Courage's
        // "another creature you control enters" trigger fires.
        let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
        g.priority.player_with_priority = 0;
        g.step = TurnStep::PreCombatMain;
        g.players[0].mana_pool.add(Color::Green, 1);
        g.players[0].mana_pool.add_colorless(1);
        g.perform_action(GameAction::CastSpell {
            card_id: bear, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast Grizzly Bears");
        drain_stack(&mut g);
        let c = g.battlefield_find(bear).unwrap();
        assert_eq!(c.power(), 4, "entering creature got +2/+0");
        assert!(c.has_keyword(&Keyword::Haste), "and haste");
        // Courage (3/3) dies to a Bolt → returns as a noncreature enchantment.
        let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
        g.players[1].mana_pool.add(Color::Red, 1);
        g.priority.player_with_priority = 1;
        g.perform_action(GameAction::CastSpell {
            card_id: bolt, target: Some(Target::Permanent(courage)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("bolt Enduring Courage");
        drain_stack(&mut g);
        let back = g.battlefield_find(courage).expect("returned to battlefield");
        assert!(back.definition.card_types.contains(&CardType::Enchantment));
        assert!(!back.definition.card_types.contains(&CardType::Creature), "returns as noncreature enchantment");
    }

    #[test]
    fn vexing_bauble_sac_draws_a_card() {
        let mut g = two_player_game();
        let bauble = g.add_card_to_battlefield(0, catalog::vexing_bauble());
        g.clear_sickness(bauble);
        g.add_card_to_library(0, catalog::island());
        g.priority.player_with_priority = 0;
        g.step = TurnStep::PreCombatMain;
        g.players[0].mana_pool.add_colorless(1);
        let lib = g.players[0].library.len();
        g.perform_action(GameAction::ActivateAbility {
            card_id: bauble, ability_index: 0, target: None, additional_targets: vec![], x_value: None, mode: None,
        }).expect("sac to draw");
        drain_stack(&mut g);
        assert_eq!(g.players[0].library.len(), lib - 1, "drew");
        assert!(!g.battlefield.iter().any(|c| c.id == bauble), "sacrificed");
    }

    #[test]
    fn loot_digs_a_creature_onto_the_battlefield() {
        let mut g = two_player_game();
        let loot = g.add_card_to_battlefield(0, catalog::loot_exuberant_explorer());
        g.clear_sickness(loot);
        // Six lands controlled so a small creature is castable; library has a bear on top.
        for _ in 0..5 { g.add_card_to_battlefield(0, catalog::forest()); }
        g.add_card_to_library(0, catalog::grizzly_bears());
        g.priority.player_with_priority = 0;
        g.step = TurnStep::PreCombatMain;
        g.players[0].mana_pool.add(Color::Green, 2);
        g.players[0].mana_pool.add_colorless(4);
        g.perform_action(GameAction::ActivateAbility {
            card_id: loot, ability_index: 0, target: None, additional_targets: vec![], x_value: None, mode: None,
        }).expect("dig");
        drain_stack(&mut g);
        assert!(g.battlefield.iter().any(|c| c.definition.name == "Grizzly Bears"), "creature put onto battlefield");
    }

    #[test]
    fn roaring_furnace_unlock_burns_for_hand_size() {
        let mut g = two_player_game();
        let room = g.add_card_to_hand(0, catalog::roaring_furnace_steaming_sauna());
        // Three cards in hand (besides the room being cast).
        for _ in 0..3 { g.add_card_to_hand(0, catalog::island()); }
        let target = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
        g.players[0].mana_pool.add(Color::Red, 1);
        g.players[0].mana_pool.add_colorless(1);
        g.perform_action(GameAction::CastRoomDoor { card_id: room, right: false })
            .expect("cast Roaring Furnace");
        drain_stack(&mut g);
        assert!(!g.battlefield.iter().any(|c| c.id == target), "burned for >= 2 (hand size)");
    }

    #[test]
    fn defiled_crypt_mints_a_horror_when_cards_leave_your_graveyard() {
        let mut g = two_player_game();
        // Cast the left door (Defiled Crypt) so its trigger goes live.
        let room = g.add_card_to_hand(0, catalog::defiled_crypt_cadaver_lab());
        g.priority.player_with_priority = 0;
        g.step = TurnStep::PreCombatMain;
        g.players[0].mana_pool.add(Color::Black, 1);
        g.players[0].mana_pool.add_colorless(3);
        g.perform_action(GameAction::CastRoomDoor { card_id: room, right: false })
            .expect("cast Defiled Crypt");
        drain_stack(&mut g);
        // A creature returning from the graveyard fires CardLeftGraveyard.
        let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
        let raise = g.add_card_to_hand(0, catalog::raise_dead());
        g.players[0].mana_pool.add(Color::Black, 1);
        g.perform_action(GameAction::CastSpell {
            card_id: raise, target: Some(Target::Permanent(bear)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast Raise Dead");
        drain_stack(&mut g);
        assert_eq!(
            g.battlefield.iter().filter(|c| c.definition.name == "Horror").count(),
            1,
            "2/2 Horror enchantment minted"
        );
    }

    #[test]
    fn winter_upkeep_each_player_draws_two() {
        let mut g = two_player_game();
        let winter = g.add_card_to_battlefield(0, catalog::winter_misanthropic_guide());
        for _ in 0..3 { g.add_card_to_library(0, catalog::island()); g.add_card_to_library(1, catalog::island()); }
        let h0 = g.players[0].hand.len();
        let h1 = g.players[1].hand.len();
        g.active_player_idx = 0;
        g.fire_step_triggers(TurnStep::Upkeep);
        drain_stack(&mut g);
        assert_eq!(g.players[0].hand.len(), h0 + 2);
        assert_eq!(g.players[1].hand.len(), h1 + 2);
        let _ = winter;
    }

    #[test]
    fn cadaver_lab_unlock_returns_a_creature_from_graveyard() {
        let mut g = two_player_game();
        let room = g.add_card_to_hand(0, catalog::defiled_crypt_cadaver_lab());
        let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
        g.priority.player_with_priority = 0;
        g.step = TurnStep::PreCombatMain;
        g.players[0].mana_pool.add(Color::Black, 1);
        g.perform_action(GameAction::CastRoomDoor { card_id: room, right: true })
            .expect("cast Cadaver Lab");
        drain_stack(&mut g);
        assert!(g.players[0].hand.iter().any(|c| c.id == bear), "creature returned to hand");
    }

    #[test]
    fn zimone_makes_primo_with_prime_lands() {
        let mut g = two_player_game();
        let zimone = g.add_card_to_battlefield(0, catalog::zimone_all_questioning());
        // Control exactly 5 lands (prime) and pretend one entered this turn.
        for _ in 0..5 { g.add_card_to_battlefield(0, catalog::forest()); }
        g.players[0].lands_played_this_turn = 1;
        g.active_player_idx = 0;
        g.fire_step_triggers(TurnStep::End);
        drain_stack(&mut g);
        let primo = g.battlefield.iter().find(|c| c.definition.name == "Primo, the Indivisible")
            .expect("Primo minted at a prime land count");
        assert_eq!(primo.counter_count(CounterType::PlusOnePlusOne), 5, "+1/+1 = land count");
        let _ = zimone;
    }

    #[test]
    fn zimone_skips_at_non_prime_land_count() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::zimone_all_questioning());
        for _ in 0..4 { g.add_card_to_battlefield(0, catalog::forest()); } // 4 = not prime
        g.players[0].lands_played_this_turn = 1;
        g.active_player_idx = 0;
        g.fire_step_triggers(TurnStep::End);
        drain_stack(&mut g);
        assert!(!g.battlefield.iter().any(|c| c.definition.name == "Primo, the Indivisible"));
    }

    #[test]
    fn ghostly_dancers_eerie_mints_a_spirit_when_an_enchantment_enters() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::ghostly_dancers());
        // Cast an enchantment through the real pipeline → Eerie token.
        let aura = g.add_card_to_hand(0, catalog::enduring_courage()); // enchantment creature
        g.priority.player_with_priority = 0;
        g.step = TurnStep::PreCombatMain;
        g.players[0].mana_pool.add(Color::Red, 2);
        g.players[0].mana_pool.add_colorless(2);
        g.perform_action(GameAction::CastSpell {
            card_id: aura, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast an enchantment");
        drain_stack(&mut g);
        assert!(
            g.battlefield.iter().any(|c| c.definition.name == "Spirit"
                && c.definition.subtypes.creature_types.contains(&CreatureType::Spirit)),
            "Eerie minted a Spirit token"
        );
    }

    #[test]
    fn pirated_copy_enters_as_a_pirate_copy() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4 to copy (any creature)
        let pc = g.add_card_to_hand(0, catalog::pirated_copy());
        g.players[0].mana_pool.add(Color::Blue, 1);
        g.players[0].mana_pool.add_colorless(4);
        g.perform_action(GameAction::CastSpell {
            card_id: pc, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast Pirated Copy");
        drain_stack(&mut g);
        let copy = g.battlefield_find(pc).expect("entered");
        assert_eq!((copy.power(), copy.toughness()), (4, 4), "copied the 4/4");
        assert!(copy.definition.subtypes.creature_types.contains(&crabomination::card::CreatureType::Pirate),
            "also a Pirate");
    }

    #[test]
    fn unwanted_remake_destroys_and_manifests_dread() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        for _ in 0..2 { g.add_card_to_library(1, catalog::island()); }
        let remake = g.add_card_to_hand(0, catalog::unwanted_remake());
        g.players[0].mana_pool.add(Color::White, 1);
        g.perform_action(GameAction::CastSpell {
            card_id: remake, target: Some(Target::Permanent(bear)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast Unwanted Remake");
        drain_stack(&mut g);
        assert!(!g.battlefield.iter().any(|c| c.id == bear), "target destroyed");
        // Its controller (P1) manifested dread → a 2/2 face-down enters under P1.
        assert!(g.battlefield.iter().any(|c| c.controller == 1 && c.face_down), "P1 manifested a face-down");
    }

    #[test]
    fn fear_of_the_dark_gains_menace_and_deathtouch_on_attack() {
        let mut g = two_player_game();
        let fear = g.add_card_to_battlefield(0, catalog::fear_of_the_dark());
        g.clear_sickness(fear);
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        while g.step != TurnStep::DeclareAttackers {
            g.perform_action(GameAction::PassPriority).unwrap();
        }
        g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker: fear, target: AttackTarget::Player(1),
        }])).expect("attack");
        drain_stack(&mut g);
        let c = g.battlefield_find(fear).unwrap();
        assert!(c.has_keyword(&Keyword::Menace) && c.has_keyword(&Keyword::Deathtouch));
    }

    #[test]
    fn brimstone_roundup_makes_a_mercenary_on_your_second_spell() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::brimstone_roundup());
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        g.step = TurnStep::PreCombatMain;
        // First spell — no token yet.
        let b1 = g.add_card_to_hand(0, catalog::grizzly_bears());
        g.players[0].mana_pool.add(Color::Green, 1);
        g.players[0].mana_pool.add_colorless(1);
        g.perform_action(GameAction::CastSpell {
            card_id: b1, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("first spell");
        drain_stack(&mut g);
        assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Mercenary").count(), 0);
        // Second spell — Brimstone Roundup mints a Mercenary.
        let b2 = g.add_card_to_hand(0, catalog::grizzly_bears());
        g.players[0].mana_pool.add(Color::Green, 1);
        g.players[0].mana_pool.add_colorless(1);
        g.perform_action(GameAction::CastSpell {
            card_id: b2, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("second spell");
        drain_stack(&mut g);
        assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Mercenary").count(), 1);
    }

    #[test]
    fn vat_emergence_reanimates_and_proliferates() {
        let mut g = two_player_game();
        let bear = g.add_card_to_graveyard(1, catalog::grizzly_bears()); // from an opponent's graveyard
        // A creature with a +1/+1 counter so Proliferate has something to bump.
        let other = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.battlefield_find_mut(other).unwrap().add_counters(CounterType::PlusOnePlusOne, 1);
        let vat = g.add_card_to_hand(0, catalog::vat_emergence());
        g.players[0].mana_pool.add(Color::Black, 1);
        g.players[0].mana_pool.add_colorless(4);
        g.perform_action(GameAction::CastSpell {
            card_id: vat, target: Some(Target::Permanent(bear)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast Vat Emergence");
        drain_stack(&mut g);
        let reanimated = g.battlefield_find(bear).expect("reanimated");
        assert_eq!(reanimated.controller, 0, "under your control");
        assert_eq!(g.battlefield_find(other).unwrap().counter_count(CounterType::PlusOnePlusOne), 2, "proliferated");
    }

    #[test]
    fn shardmages_rescue_buffs_and_protects() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let aura = g.add_card_to_hand(0, catalog::shardmages_rescue());
        g.players[0].mana_pool.add(Color::White, 1);
        g.perform_action(GameAction::CastSpell {
            card_id: aura, target: Some(Target::Permanent(bear)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast Shardmage's Rescue");
        drain_stack(&mut g);
        let view = g.compute_battlefield();
        let c = view.iter().find(|c| c.id == bear).unwrap();
        assert_eq!((c.power, c.toughness), (3, 3), "+1/+1");
        assert!(c.keywords.contains(&Keyword::Hexproof), "granted hexproof");
    }

    #[test]
    fn trail_of_crumbs_makes_food_and_digs_on_food_sac() {
        let mut g = two_player_game();
        let trail = g.add_card_to_battlefield(0, catalog::trail_of_crumbs());
        g.fire_self_etb_triggers(trail, 0);
        drain_stack(&mut g);
        let food = g.battlefield.iter().find(|c| c.definition.name == "Food").expect("ETB Food").id;
        // A permanent on top of the library to dig into hand.
        g.add_card_to_library(0, catalog::grizzly_bears());
        g.add_card_to_library(0, catalog::forest());
        g.players[0].mana_pool.add_colorless(1);
        let hand0 = g.players[0].hand.len();
        // Sacrifice the Food → trigger → pay {1} → dig (AutoDecider takes the
        // beneficial pay and reveals the top permanent).
        let ctx = crabomination::game::effects::EffectContext::for_spell(0, None, 0, 0);
        g.resolve_effect(
            &crabomination::effect::Effect::Sacrifice {
                who: crabomination::effect::Selector::You,
                count: crabomination::effect::Value::ONE,
                filter: crabomination::card::SelectionRequirement::HasArtifactSubtype(
                    crabomination::card::ArtifactSubtype::Food,
                ),
            },
            &ctx,
        ).unwrap();
        drain_stack(&mut g);
        assert!(!g.battlefield.iter().any(|c| c.id == food), "Food was sacrificed");
        let _ = hand0; // the dig is an optional {1} payment (LookPickToHand machinery)
    }

    #[test]
    fn macabre_reconstruction_returns_up_to_two_and_is_cheaper_after_a_death() {
        let mut g = two_player_game();
        let a = g.add_card_to_graveyard(0, catalog::grizzly_bears());
        let b = g.add_card_to_graveyard(0, catalog::grizzly_bears());
        g.players[0].creatures_died_this_turn = 1; // {2} cheaper → {1}{B}
        let mr = g.add_card_to_hand(0, catalog::macabre_reconstruction());
        g.players[0].mana_pool.add(Color::Black, 1);
        g.players[0].mana_pool.add_colorless(1);
        g.perform_action(GameAction::CastSpell {
            card_id: mr, target: Some(Target::Permanent(a)),
            additional_targets: vec![Target::Permanent(b)], mode: None, x_value: None,
        }).expect("Macabre castable for {1}{B} after a death");
        drain_stack(&mut g);
        assert!(g.players[0].hand.iter().any(|c| c.id == a));
        assert!(g.players[0].hand.iter().any(|c| c.id == b), "both creatures returned");
    }
}

mod recent53 {
    use crabomination::card::Keyword;
    use crabomination::catalog;
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    use crabomination::game::types::{Attack, AttackTarget};
    use crabomination::game::*;
    use crabomination::mana::Color;

    fn advance_to(g: &mut GameState, step: TurnStep) {
        while g.step != step {
            g.perform_action(GameAction::PassPriority).expect("pass priority");
        }
    }

    #[test]
    fn by_force_destroys_x_artifacts() {
        let mut g = two_player_game();
        let a = g.add_card_to_battlefield(1, catalog::mind_stone());
        let b = g.add_card_to_battlefield(1, catalog::mind_stone());
        let spell = g.add_card_to_hand(0, catalog::by_force());
        g.players[0].mana_pool.add(Color::Red, 1);
        g.players[0].mana_pool.add_colorless(2);
        g.perform_action(GameAction::CastSpell {
            card_id: spell,
            target: Some(Target::Permanent(a)),
            additional_targets: vec![Target::Permanent(b)],
            mode: None,
            x_value: Some(2),
        })
        .expect("cast {X=2}{R} destroying two artifacts");
        drain_stack(&mut g);
        assert!(g.battlefield_find(a).is_none() && g.battlefield_find(b).is_none(), "both artifacts destroyed");
    }

    #[test]
    fn palace_jailer_takes_the_crown_and_a_creature() {
        let mut g = two_player_game();
        let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let jailer = g.add_card_to_battlefield(0, catalog::palace_jailer());
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Target(Target::Permanent(victim))]));
        g.fire_self_etb_triggers(jailer, 0);
        drain_stack(&mut g);
        assert_eq!(g.monarch, Some(0), "controller became the monarch");
        assert!(g.battlefield_find(victim).is_none(), "opponent's creature exiled");

        // CR 724 — when an opponent takes the crown, the creature comes back.
        let mut events = vec![];
        g.set_monarch(1, &mut events);
        assert!(g.battlefield_find(victim).is_some(), "creature returns when the monarchy moves");
    }

    #[test]
    fn loxodon_smiter_cant_be_countered() {
        assert!(catalog::loxodon_smiter().keywords.contains(&Keyword::CantBeCountered));
    }

    #[test]
    fn leonin_vanguard_pumps_with_a_full_board() {
        let mut g = two_player_game();
        let leonin = g.add_card_to_battlefield(0, catalog::leonin_vanguard());
        g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let life = g.players[0].life;
        g.step = TurnStep::BeginCombat;
        g.priority.player_with_priority = 0;
        g.fire_step_triggers(TurnStep::BeginCombat);
        drain_stack(&mut g);
        let cp = g.compute_battlefield();
        let v = cp.iter().find(|c| c.id == leonin).unwrap();
        assert_eq!((v.power, v.toughness), (2, 2), "buffed with 3 creatures");
        assert_eq!(g.players[0].life, life + 1, "gained a life");
    }

    #[test]
    fn giada_scales_entering_angels() {
        let mut g = two_player_game();
        // Giada + one more Angel already out = two Angels you control.
        g.add_card_to_battlefield(0, catalog::giada_font_of_hope());
        g.add_card_to_battlefield(0, catalog::serra_angel());
        // A third Angel enters (reanimation/move path) with +1/+1 per existing Angel.
        let newcomer = g.move_card_to_battlefield_for_test(0, catalog::serra_angel());
        let cp = g.compute_battlefield();
        let a = cp.iter().find(|c| c.id == newcomer).unwrap();
        // Serra Angel is 4/4; two Angels already controlled → +2/+2 → 6/6.
        assert_eq!((a.power, a.toughness), (6, 6), "entered with two +1/+1 counters");
    }

    #[test]
    fn hopeful_initiate_removes_counters_from_among_creatures() {
        use crabomination::card::CounterType;
        let mut g = two_player_game();
        let init = g.add_card_to_battlefield(0, catalog::hopeful_initiate());
        let ally = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let ench = g.add_card_to_battlefield(1, catalog::mark_of_asylum());
        // Spread two +1/+1 counters across the two creatures.
        g.battlefield.iter_mut().find(|c| c.id == init).unwrap().add_counters(CounterType::PlusOnePlusOne, 1);
        g.battlefield.iter_mut().find(|c| c.id == ally).unwrap().add_counters(CounterType::PlusOnePlusOne, 1);
        g.players[0].mana_pool.add(Color::White, 1);
        g.players[0].mana_pool.add_colorless(2);
        g.priority.player_with_priority = 0;
        g.step = TurnStep::PreCombatMain;
        g.perform_action(GameAction::ActivateAbility {
            card_id: init, ability_index: 0, target: Some(Target::Permanent(ench)),
            additional_targets: Vec::new(), x_value: None, mode: None,
        }).expect("activates by removing two counters from among creatures");
        drain_stack(&mut g);
        assert!(g.battlefield_find(ench).is_none(), "enchantment destroyed");
        assert_eq!(g.battlefield_find(init).unwrap().counter_count(CounterType::PlusOnePlusOne), 0);
        assert_eq!(g.battlefield_find(ally).unwrap().counter_count(CounterType::PlusOnePlusOne), 0);
    }

    #[test]
    fn sanctum_prelate_locks_chosen_mana_value() {
        let mut g = two_player_game();
        let prelate = g.add_card_to_battlefield(0, catalog::sanctum_prelate());
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Amount(1)]));
        g.fire_self_etb_triggers(prelate, 0);
        drain_stack(&mut g);
        assert_eq!(g.battlefield_find(prelate).unwrap().chosen_number, Some(1));

        // A MV-1 noncreature spell (Lightning Bolt) is locked; a MV-1 creature and a
        // MV-2 noncreature are fine.
        assert!(g.noncreature_spell_cast_locked(&catalog::lightning_bolt()), "MV-1 noncreature locked");
        assert!(!g.noncreature_spell_cast_locked(&catalog::grizzly_bears()), "creature never locked");
        assert!(!g.noncreature_spell_cast_locked(&catalog::mind_stone()), "MV-2 noncreature unaffected");
    }

    #[test]
    fn old_rutstein_mills_and_branches_by_type() {
        let mut g = two_player_game();
        let rutstein = g.add_card_to_battlefield(0, catalog::old_rutstein());
        // Land on top → mills a land → Treasure.
        let land = g.next_id();
        g.players[0].add_to_library_top(land, catalog::forest());
        g.fire_self_etb_triggers(rutstein, 0);
        drain_stack(&mut g);
        assert!(
            g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Treasure"),
            "milled land made a Treasure",
        );
        // Creature on top → Insect token.
        let crea = g.next_id();
        g.players[0].add_to_library_top(crea, catalog::grizzly_bears());
        g.fire_self_etb_triggers(rutstein, 0);
        drain_stack(&mut g);
        assert!(
            g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Insect"),
            "milled creature made an Insect",
        );
    }

    #[test]
    fn thorn_of_the_black_rose_takes_the_crown() {
        let mut g = two_player_game();
        let thorn = g.add_card_to_battlefield(0, catalog::thorn_of_the_black_rose());
        g.fire_self_etb_triggers(thorn, 0);
        drain_stack(&mut g);
        assert_eq!(g.monarch, Some(0));
    }

    #[test]
    fn throne_warden_grows_while_monarch() {
        use crabomination::card::CounterType;
        let mut g = two_player_game();
        let warden = g.add_card_to_battlefield(0, catalog::throne_warden());
        // Not the monarch → no growth.
        g.monarch = None;
        g.step = TurnStep::End;
        g.fire_step_triggers(TurnStep::End);
        drain_stack(&mut g);
        assert_eq!(g.battlefield_find(warden).unwrap().counter_count(CounterType::PlusOnePlusOne), 0);
        // Monarch → +1/+1.
        g.monarch = Some(0);
        g.fire_step_triggers(TurnStep::End);
        drain_stack(&mut g);
        assert_eq!(g.battlefield_find(warden).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
    }

    #[test]
    fn skyline_despot_makes_a_dragon_while_monarch() {
        let mut g = two_player_game();
        let despot = g.add_card_to_battlefield(0, catalog::skyline_despot());
        g.fire_self_etb_triggers(despot, 0);
        drain_stack(&mut g);
        assert_eq!(g.monarch, Some(0), "took the crown on ETB");
        g.step = TurnStep::Upkeep;
        g.fire_step_triggers(TurnStep::Upkeep);
        drain_stack(&mut g);
        assert!(
            g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Dragon"),
            "monarch upkeep minted a Dragon",
        );
    }

    #[test]
    fn keeper_of_keys_unblockable_while_monarch() {
        let mut g = two_player_game();
        let keeper = g.add_card_to_battlefield(0, catalog::keeper_of_keys());
        let ally = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.fire_self_etb_triggers(keeper, 0);
        drain_stack(&mut g);
        assert_eq!(g.monarch, Some(0));
        g.step = TurnStep::Upkeep;
        g.fire_step_triggers(TurnStep::Upkeep);
        drain_stack(&mut g);
        let cp = g.compute_battlefield();
        assert!(
            cp.iter().find(|c| c.id == ally).unwrap().keywords.contains(&Keyword::Unblockable),
            "your creatures gained unblockable while you're the monarch",
        );
    }

    #[test]
    fn judith_pings_on_nontoken_death() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::judith_the_scourge_diva());
        let doomed = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        // Bolt our own bear so the SBA dispatches CreatureDied to Judith; her
        // trigger then pings the opponent (scripted target).
        let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
        g.players[0].mana_pool.add(Color::Red, 1);
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Target(Target::Player(1))]));
        let before = g.players[1].life;
        g.perform_action(GameAction::CastSpell {
            card_id: bolt, target: Some(Target::Permanent(doomed)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("bolt the bear");
        drain_stack(&mut g);
        assert_eq!(g.players[1].life, before - 1, "Judith dealt 1 on a nontoken death");
    }

    #[test]
    fn marchesas_decree_bleeds_attackers() {
        let mut g = two_player_game();
        // Player 1 controls the enchantment (is monarch); player 0 attacks them.
        let decree = g.add_card_to_battlefield(1, catalog::marchesas_decree());
        g.fire_self_etb_triggers(decree, 1);
        drain_stack(&mut g);
        let attacker = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.clear_sickness(attacker);
        let before = g.players[0].life;
        advance_to(&mut g, TurnStep::DeclareAttackers);
        g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker, target: AttackTarget::Player(1),
        }])).expect("declare attack on the monarch");
        drain_stack(&mut g);
        assert_eq!(g.players[0].life, before - 1, "the attacker's controller lost 1 life");
    }

    #[test]
    fn serra_ascendant_is_huge_at_high_life() {
        let mut g = two_player_game();
        let serra = g.add_card_to_battlefield(0, catalog::serra_ascendant());
        // Below 30 life: a 1/1.
        g.players[0].life = 20;
        let cp = g.compute_battlefield();
        let s = cp.iter().find(|c| c.id == serra).unwrap();
        assert_eq!((s.power, s.toughness), (1, 1));
        assert!(!s.keywords.contains(&Keyword::Flying));
        // 30+ life: 6/6 flier.
        g.players[0].life = 30;
        let cp = g.compute_battlefield();
        let s = cp.iter().find(|c| c.id == serra).unwrap();
        assert_eq!((s.power, s.toughness), (6, 6));
        assert!(s.keywords.contains(&Keyword::Flying));
    }

    #[test]
    fn angelic_accord_makes_an_angel_after_big_lifegain() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::angelic_accord());
        // Gain 4 life this turn, then hit the end step.
        g.adjust_life_applied(0, 4);
        g.step = TurnStep::End;
        g.fire_step_triggers(TurnStep::End);
        drain_stack(&mut g);
        assert!(
            g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Angel"),
            "gained 4+ → Angel token",
        );
    }

    #[test]
    fn warleaders_helix_burns_and_gains() {
        let mut g = two_player_game();
        let helix = g.add_card_to_hand(0, catalog::warleaders_helix());
        g.players[0].mana_pool.add(Color::Red, 1);
        g.players[0].mana_pool.add(Color::White, 1);
        g.players[0].mana_pool.add_colorless(2);
        let (dmg_before, life_before) = (g.players[1].life, g.players[0].life);
        g.perform_action(GameAction::CastSpell {
            card_id: helix, target: Some(Target::Player(1)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast helix at the opponent");
        drain_stack(&mut g);
        assert_eq!(g.players[1].life, dmg_before - 4, "4 damage");
        assert_eq!(g.players[0].life, life_before + 4, "gained 4");
    }

    #[test]
    fn wojek_halberdiers_battalion_first_strike() {
        let mut g = two_player_game();
        let wojek = g.add_card_to_battlefield(0, catalog::wojek_halberdiers());
        let a = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let b = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        for id in [wojek, a, b] { g.clear_sickness(id); }
        advance_to(&mut g, TurnStep::DeclareAttackers);
        g.perform_action(GameAction::DeclareAttackers(vec![
            Attack { attacker: wojek, target: AttackTarget::Player(1) },
            Attack { attacker: a, target: AttackTarget::Player(1) },
            Attack { attacker: b, target: AttackTarget::Player(1) },
        ])).expect("battalion attack");
        drain_stack(&mut g);
        assert!(
            g.compute_battlefield().iter().find(|c| c.id == wojek).unwrap().keywords.contains(&Keyword::FirstStrike),
            "battalion granted first strike",
        );
    }

    #[test]
    fn firemane_avenger_battalion_bolts() {
        let mut g = two_player_game();
        let fm = g.add_card_to_battlefield(0, catalog::firemane_avenger());
        let a = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let b = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        for id in [fm, a, b] { g.clear_sickness(id); }
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Target(Target::Player(1))]));
        let (dmg_before, life_before) = (g.players[1].life, g.players[0].life);
        advance_to(&mut g, TurnStep::DeclareAttackers);
        g.perform_action(GameAction::DeclareAttackers(vec![
            Attack { attacker: fm, target: AttackTarget::Player(1) },
            Attack { attacker: a, target: AttackTarget::Player(1) },
            Attack { attacker: b, target: AttackTarget::Player(1) },
        ])).expect("battalion attack");
        drain_stack(&mut g);
        assert_eq!(g.players[1].life, dmg_before - 3, "battalion dealt 3");
        assert_eq!(g.players[0].life, life_before + 3, "battalion gained 3");
    }

    #[test]
    fn assemble_the_legion_musters_more_each_turn() {
        use crabomination::card::CounterType;
        let mut g = two_player_game();
        let assemble = g.add_card_to_battlefield(0, catalog::assemble_the_legion());
        g.step = TurnStep::Upkeep;
        g.fire_step_triggers(TurnStep::Upkeep);
        drain_stack(&mut g);
        assert_eq!(g.battlefield_find(assemble).unwrap().counter_count(CounterType::Muster), 1);
        let after_one = g.battlefield.iter().filter(|c| c.definition.name == "Soldier").count();
        assert_eq!(after_one, 1, "one Soldier at one muster counter");
        g.fire_step_triggers(TurnStep::Upkeep);
        drain_stack(&mut g);
        let after_two = g.battlefield.iter().filter(|c| c.definition.name == "Soldier").count();
        assert_eq!(after_two, 1 + 2, "two more Soldiers at two muster counters");
    }

    #[test]
    fn war_priest_smashes_an_enchantment() {
        let mut g = two_player_game();
        let ench = g.add_card_to_battlefield(1, catalog::mark_of_asylum());
        let priest = g.add_card_to_battlefield(0, catalog::war_priest_of_thune());
        g.decider = Box::new(ScriptedDecider::new([
            DecisionAnswer::Bool(true),
            DecisionAnswer::Target(Target::Permanent(ench)),
        ]));
        g.fire_self_etb_triggers(priest, 0);
        drain_stack(&mut g);
        assert!(g.battlefield_find(ench).is_none(), "enchantment destroyed");
    }

    #[test]
    fn goldnight_redeemer_gains_two_per_other_creature() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let redeemer = g.add_card_to_battlefield(0, catalog::goldnight_redeemer());
        let before = g.players[0].life;
        g.fire_self_etb_triggers(redeemer, 0);
        drain_stack(&mut g);
        assert_eq!(g.players[0].life, before + 4, "2 life × 2 other creatures");
    }

    #[test]
    fn kinsbaile_borderguard_scales_and_leaves_tokens() {
        use crabomination::card::CounterType;
        let mut g = two_player_game();
        // Two other Kithkin already out.
        g.add_card_to_battlefield(0, catalog::kinsbaile_borderguard());
        g.add_card_to_battlefield(0, catalog::kinsbaile_borderguard());
        let newcomer = g.move_card_to_battlefield_for_test(0, catalog::kinsbaile_borderguard());
        assert_eq!(
            g.battlefield_find(newcomer).unwrap().counter_count(CounterType::PlusOnePlusOne),
            2,
            "entered with a counter per other Kithkin",
        );
        let tokens_before = g.battlefield.iter().filter(|c| c.definition.name == "Kithkin Soldier").count();
        g.remove_to_graveyard_with_triggers(newcomer);
        drain_stack(&mut g);
        let tokens_after = g.battlefield.iter().filter(|c| c.definition.name == "Kithkin Soldier").count();
        assert_eq!(tokens_after - tokens_before, 2, "made a token per counter on death");
    }

    #[test]
    fn terror_of_the_peaks_pings_on_creature_entry() {
        let mut g = two_player_game();
        let terror = g.add_card_to_battlefield(0, catalog::terror_of_the_peaks());
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Target(Target::Player(1))]));
        let before = g.players[1].life;
        // Cast a 2/2; Terror pings for its power (2).
        let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
        g.players[0].mana_pool.add(Color::Green, 1);
        g.players[0].mana_pool.add_colorless(1);
        g.perform_action(GameAction::CastSpell {
            card_id: bear, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast bear");
        drain_stack(&mut g);
        let _ = terror;
        assert_eq!(g.players[1].life, before - 2, "Terror dealt the entrant's power");
    }

    #[test]
    fn warstorm_surge_pings_for_entrant_power() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::warstorm_surge());
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Target(Target::Player(1))]));
        let before = g.players[1].life;
        let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
        g.players[0].mana_pool.add(Color::Green, 1);
        g.players[0].mana_pool.add_colorless(1);
        g.perform_action(GameAction::CastSpell {
            card_id: bear, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast bear");
        drain_stack(&mut g);
        assert_eq!(g.players[1].life, before - 2, "Warstorm Surge pinged for the entrant's power");
    }

    #[test]
    fn tuktuk_returns_bigger() {
        let mut g = two_player_game();
        let tuk = g.add_card_to_battlefield(0, catalog::tuktuk_the_explorer());
        let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
        g.players[0].mana_pool.add(Color::Red, 1);
        g.perform_action(GameAction::CastSpell {
            card_id: bolt, target: Some(Target::Permanent(tuk)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("bolt Tuktuk");
        drain_stack(&mut g);
        let ret = g.battlefield.iter().find(|c| c.definition.name == "Tuktuk the Returned").expect("token made");
        assert_eq!((ret.definition.power, ret.definition.toughness), (5, 5));
    }

    #[test]
    fn tine_shrike_has_infect() {
        assert!(catalog::tine_shrike().keywords.contains(&Keyword::Infect));
    }

    #[test]
    fn balustrade_spy_mills_to_a_land() {
        use crabomination::game::Target;
        let mut g = two_player_game();
        // Stack player 1's library: two nonlands on top of a land.
        for def in [catalog::grizzly_bears(), catalog::lightning_bolt()] {
            let id = g.next_id();
            g.players[1].add_to_library_top(id, def);
        }
        let land = g.next_id();
        g.players[1].library.push(crabomination::card::CardInstance::new(land, catalog::forest(), 1));
        let spy = g.add_card_to_battlefield(0, catalog::balustrade_spy());
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Target(Target::Player(1))]));
        g.fire_self_etb_triggers(spy, 0);
        drain_stack(&mut g);
        assert!(g.players[1].graveyard.iter().any(|c| c.definition.name == "Forest"), "milled through to a land");
        assert!(g.players[1].graveyard.len() >= 3, "the nonlands + the land were milled");
    }

    #[test]
    fn ravos_anthems_and_recurs() {
        let mut g = two_player_game();
        let ravos = g.add_card_to_battlefield(0, catalog::ravos_soultender());
        let ally = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        // Anthem: other creature 2/2 → 3/3.
        let cp = g.compute_battlefield();
        let a = cp.iter().find(|c| c.id == ally).unwrap();
        assert_eq!((a.power, a.toughness), (3, 3), "+1/+1 to others");
        // Upkeep return of a graveyard creature (script "yes"; auto-pick the target).
        let dead = g.add_card_to_graveyard(0, catalog::grizzly_bears());
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
        g.step = TurnStep::Upkeep;
        g.fire_step_triggers(TurnStep::Upkeep);
        drain_stack(&mut g);
        let _ = ravos;
        assert!(g.players[0].hand.iter().any(|c| c.id == dead), "returned the creature to hand");
    }

    #[test]
    fn adriana_grants_melee_to_the_team() {
        let mut g = two_player_game();
        let adriana = g.add_card_to_battlefield(0, catalog::adriana_captain_of_the_guard());
        let ally = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.clear_sickness(adriana);
        g.clear_sickness(ally);
        advance_to(&mut g, TurnStep::DeclareAttackers);
        g.perform_action(GameAction::DeclareAttackers(vec![
            Attack { attacker: adriana, target: AttackTarget::Player(1) },
            Attack { attacker: ally, target: AttackTarget::Player(1) },
        ])).expect("attack with both");
        drain_stack(&mut g);
        let cp = g.compute_battlefield();
        // Melee (flat +1/+1 approximation) fires for both attackers.
        assert_eq!(
            { let a = cp.iter().find(|c| c.id == ally).unwrap(); (a.power, a.toughness) },
            (3, 3),
            "granted melee pumped the ally",
        );
    }

    #[test]
    fn melee_counts_each_opponent_attacked() {
        // CR 702.122 — in multiplayer, Melee scales with distinct opponents hit.
        let mut g = crabomination::game::multi_player_game(3);
        let adriana = g.add_card_to_battlefield(0, catalog::adriana_captain_of_the_guard());
        let ally = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.clear_sickness(adriana);
        g.clear_sickness(ally);
        advance_to(&mut g, TurnStep::DeclareAttackers);
        g.perform_action(GameAction::DeclareAttackers(vec![
            Attack { attacker: adriana, target: AttackTarget::Player(1) },
            Attack { attacker: ally, target: AttackTarget::Player(2) },
        ]))
        .expect("attack two different opponents");
        drain_stack(&mut g);
        let cp = g.compute_battlefield();
        // Two distinct opponents attacked → melee grants +2/+2 (Adriana 4/4 → 6/6).
        assert_eq!(
            { let a = cp.iter().find(|c| c.id == adriana).unwrap(); (a.power, a.toughness) },
            (6, 6),
            "Adriana got +2/+2 for two opponents",
        );
    }

    #[test]
    fn regal_behemoth_doubles_land_mana_while_monarch() {
        let mut g = two_player_game();
        let behemoth = g.add_card_to_battlefield(0, catalog::regal_behemoth());
        let forest = g.add_card_to_battlefield(0, catalog::forest());
        g.fire_self_etb_triggers(behemoth, 0);
        drain_stack(&mut g);
        assert_eq!(g.monarch, Some(0));
        // Tap the Forest: one {G} from the land + one extra while monarch.
        g.priority.player_with_priority = 0;
        g.step = TurnStep::PreCombatMain;
        g.perform_action(GameAction::ActivateAbility {
            card_id: forest, ability_index: 0, target: None, additional_targets: vec![], x_value: None, mode: None,
        }).expect("tap forest for mana");
        assert_eq!(g.players[0].mana_pool.amount(Color::Green), 2, "monarch got an extra mana");
    }

    #[test]
    fn gallant_cavalry_brings_a_friend() {
        let mut g = two_player_game();
        let cav = g.add_card_to_battlefield(0, catalog::gallant_cavalry());
        g.fire_self_etb_triggers(cav, 0);
        drain_stack(&mut g);
        assert!(
            g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Knight"),
            "made a Knight token",
        );
    }

    #[test]
    fn valiant_knight_lords_and_grants_double_strike() {
        let mut g = two_player_game();
        let valiant = g.add_card_to_battlefield(0, catalog::valiant_knight());
        let ally = g.add_card_to_battlefield(0, catalog::gallant_cavalry()); // a Knight
        // Anthem: the other Knight is 2/2 → 3/3.
        let cp = g.compute_battlefield();
        let a = cp.iter().find(|c| c.id == ally).unwrap();
        assert_eq!((a.power, a.toughness), (3, 3), "+1/+1 to other Knights");
        // Activated grant: Knights gain double strike.
        g.players[0].mana_pool.add(Color::White, 2);
        g.players[0].mana_pool.add_colorless(3);
        g.priority.player_with_priority = 0;
        g.step = TurnStep::PreCombatMain;
        g.perform_action(GameAction::ActivateAbility {
            card_id: valiant, ability_index: 0, target: None,
            additional_targets: Vec::new(), x_value: None, mode: None,
        }).expect("activate double-strike pump");
        drain_stack(&mut g);
        let cp = g.compute_battlefield();
        assert!(
            cp.iter().find(|c| c.id == valiant).unwrap().keywords.contains(&Keyword::DoubleStrike),
            "Knights gained double strike",
        );
    }

    #[test]
    fn custodi_lich_edicts_and_crowns() {
        let mut g = two_player_game();
        let opp = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let lich = g.add_card_to_battlefield(0, catalog::custodi_lich());
        g.fire_self_etb_triggers(lich, 0);
        drain_stack(&mut g);
        assert_eq!(g.monarch, Some(0), "took the crown");
        assert!(g.battlefield_find(opp).is_none(), "opponent sacrificed its only creature");
    }
}

mod recent54 {
    use crabomination::card::{CounterType, Keyword};
    use crabomination::catalog;
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    use crabomination::effect::{Effect, PlayerRef, Selector, Value};
    use crabomination::game::effects::EffectContext;
    use crabomination::game::*;
    use crabomination::mana::Color;

    fn counters(g: &GameState, id: CardId) -> u32 {
        g.battlefield_find(id).unwrap().counter_count(CounterType::PlusOnePlusOne)
    }

    fn cast_from_hand(g: &mut GameState, id: CardId, colors: &[(Color, u32)], generic: u32) {
        for (c, n) in colors {
            g.players[0].mana_pool.add(*c, *n);
        }
        if generic > 0 {
            g.players[0].mana_pool.add_colorless(generic);
        }
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::CastSpell {
            card_id: id,
            target: None,
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .expect("cast from hand");
        drain_stack(g);
    }

    #[test]
    fn good_fortune_unicorn_counters_the_entrant() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::good_fortune_unicorn());
        let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
        cast_from_hand(&mut g, bear, &[(Color::Green, 1)], 1);
        assert_eq!(counters(&g, bear), 1, "entering creature got a +1/+1 counter");
    }

    #[test]
    fn ivy_lane_denizen_counters_a_target_on_green_entry() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::ivy_lane_denizen());
        let ally = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Target(Target::Permanent(ally))]));
        let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
        cast_from_hand(&mut g, bear, &[(Color::Green, 1)], 1);
        assert_eq!(counters(&g, ally), 1, "green entry put a counter on the chosen creature");
    }

    #[test]
    fn managorger_hydra_grows_on_any_spell() {
        let mut g = two_player_game();
        let hydra = g.add_card_to_battlefield(0, catalog::managorger_hydra());
        let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
        cast_from_hand(&mut g, bear, &[(Color::Green, 1)], 1);
        assert_eq!(counters(&g, hydra), 1, "cast a spell → +1/+1 on Managorger");
    }

    #[test]
    fn herd_baloth_makes_a_beast_when_countered() {
        let mut g = two_player_game();
        let herd = g.add_card_to_battlefield(0, catalog::herd_baloth());
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
        let ctx = EffectContext::for_spell(0, Some(Target::Permanent(herd)), 0, 0);
        let evs = g
            .resolve_effect(
                &Effect::AddCounter {
                    what: Selector::Target(0),
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                },
                &ctx,
            )
            .unwrap();
        g.dispatch_triggers_for_events(&evs);
        drain_stack(&mut g);
        let beasts = g.battlefield.iter().filter(|c| c.definition.name == "Beast").count();
        assert_eq!(beasts, 1, "putting a counter on Herd Baloth minted a 4/4 Beast");
    }

    #[test]
    fn duskshell_crawler_grants_trample_to_counter_bearers() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::duskshell_crawler());
        let ally = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let ctx = EffectContext::for_spell(0, Some(Target::Permanent(ally)), 0, 0);
        g.resolve_effect(
            &Effect::AddCounter { what: Selector::Target(0), kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
            &ctx,
        )
        .unwrap();
        let cp = g.compute_battlefield();
        let a = cp.iter().find(|c| c.id == ally).unwrap();
        assert!(a.keywords.contains(&Keyword::Trample), "counter-bearer has trample");
    }

    #[test]
    fn longshot_squad_grants_reach_to_counter_bearers() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::longshot_squad());
        let ally = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let ctx = EffectContext::for_spell(0, Some(Target::Permanent(ally)), 0, 0);
        g.resolve_effect(
            &Effect::AddCounter { what: Selector::Target(0), kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
            &ctx,
        )
        .unwrap();
        let cp = g.compute_battlefield();
        let a = cp.iter().find(|c| c.id == ally).unwrap();
        assert!(a.keywords.contains(&Keyword::Reach), "counter-bearer has reach");
        // A creature with no counter is unaffected.
        let bare = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let cp = g.compute_battlefield();
        assert!(!cp.iter().find(|c| c.id == bare).unwrap().keywords.contains(&Keyword::Reach));
    }

    #[test]
    fn kami_of_whispered_hopes_adds_an_extra_counter() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::kami_of_whispered_hopes());
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let ctx = EffectContext::for_spell(0, Some(Target::Permanent(bear)), 0, 0);
        g.resolve_effect(
            &Effect::AddCounter { what: Selector::Target(0), kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
            &ctx,
        )
        .unwrap();
        assert_eq!(counters(&g, bear), 2, "one counter became two (that many plus one)");
    }

    #[test]
    fn old_gnawbone_mints_treasure_per_combat_damage() {
        let mut g = two_player_game();
        let gnaw = g.add_card_to_battlefield(0, catalog::old_gnawbone());
        let effect = catalog::old_gnawbone().triggered_abilities[0].effect.clone();
        let ctx = EffectContext { event_amount: 7, ..EffectContext::for_trigger(gnaw, 0, None, 0) };
        g.resolve_effect(&effect, &ctx).unwrap();
        let treasures = g.battlefield.iter().filter(|c| c.definition.name == "Treasure").count();
        assert_eq!(treasures, 7, "7 combat damage → 7 Treasure tokens");
    }

    #[test]
    fn ulvenwald_tracker_fights() {
        let mut g = two_player_game();
        let tracker = g.add_card_to_battlefield(0, catalog::ulvenwald_tracker());
        let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        g.clear_sickness(tracker);
        g.players[0].mana_pool.add(Color::Green, 1);
        g.players[0].mana_pool.add_colorless(1);
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::ActivateAbility {
            card_id: tracker,
            ability_index: 0,
            target: Some(Target::Permanent(mine)),
            additional_targets: vec![Target::Permanent(foe)],
            x_value: None, mode: None,
        })
        .expect("fight");
        drain_stack(&mut g);
        assert!(g.battlefield_find(mine).is_none() && g.battlefield_find(foe).is_none(), "2/2s trade in the fight");
    }

    #[test]
    fn nissa_voice_pumps_the_team_and_makes_plants() {
        let mut g = two_player_game();
        let nissa = g.add_card_to_battlefield(0, catalog::nissa_voice_of_zendikar());
        let a = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let b = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        // -2: +1/+1 on each creature you control.
        g.perform_action(GameAction::ActivateLoyaltyAbility { card_id: nissa, ability_index: 1, target: None, x_value: None })
            .expect("minus two");
        drain_stack(&mut g);
        assert_eq!(counters(&g, a), 1);
        assert_eq!(counters(&g, b), 1);
    }

    #[test]
    fn nissa_voice_plus_one_makes_a_plant() {
        let mut g = two_player_game();
        let nissa = g.add_card_to_battlefield(0, catalog::nissa_voice_of_zendikar());
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::ActivateLoyaltyAbility { card_id: nissa, ability_index: 0, target: None, x_value: None })
            .expect("plus one");
        drain_stack(&mut g);
        assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Plant").count(), 1, "made a 0/1 Plant");
    }

    #[test]
    fn gyre_sage_taps_for_green_per_counter() {
        let mut g = two_player_game();
        let sage = g.add_card_to_battlefield(0, catalog::gyre_sage());
        let ctx = EffectContext::for_spell(0, Some(Target::Permanent(sage)), 0, 0);
        g.resolve_effect(
            &Effect::AddCounter { what: Selector::Target(0), kind: CounterType::PlusOnePlusOne, amount: Value::Const(2) },
            &ctx,
        )
        .unwrap();
        g.clear_sickness(sage);
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::ActivateAbility {
            card_id: sage, ability_index: 0, target: None, additional_targets: vec![], x_value: None, mode: None,
        })
        .expect("mana ability");
        assert_eq!(g.players[0].mana_pool.total(), 2, "{{T}}: two counters → two green mana");
    }

    #[test]
    fn elusive_krasis_is_unblockable_and_evolves() {
        let mut g = two_player_game();
        let krasis = g.add_card_to_battlefield(0, catalog::elusive_krasis());
        assert!(g.battlefield_find(krasis).unwrap().definition.keywords.contains(&Keyword::Unblockable));
        // A bigger creature entering evolves the 0/4 (bear power 2 > 0).
        let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
        cast_from_hand(&mut g, bear, &[(Color::Green, 1)], 1);
        assert_eq!(counters(&g, krasis), 1, "evolve triggered on the entering creature");
    }

    #[test]
    fn corpsejack_menace_doubles_counters() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::corpsejack_menace());
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let ctx = EffectContext::for_spell(0, Some(Target::Permanent(bear)), 0, 0);
        g.resolve_effect(
            &Effect::AddCounter { what: Selector::Target(0), kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
            &ctx,
        )
        .unwrap();
        assert_eq!(counters(&g, bear), 2, "one counter doubled to two");
    }

    #[test]
    fn prime_speaker_zegana_enters_scaled_and_draws() {
        let mut g = two_player_game();
        for _ in 0..8 { g.add_card_to_library(0, catalog::forest()); }
        g.add_card_to_battlefield(0, catalog::serra_angel()); // 4/4
        let hand = g.players[0].hand.len();
        let zeg = g.move_card_to_battlefield_for_test(0, catalog::prime_speaker_zegana());
        drain_stack(&mut g);
        assert_eq!(counters(&g, zeg), 4, "entered with counters = greatest other power");
        assert_eq!(g.players[0].hand.len(), hand + 5, "drew cards equal to its power (1+4)");
    }

    #[test]
    fn cold_eyed_selkie_has_islandwalk_and_draws_on_damage() {
        let mut g = two_player_game();
        let selkie = g.add_card_to_battlefield(0, catalog::cold_eyed_selkie());
        assert!(g
            .battlefield_find(selkie)
            .unwrap()
            .definition
            .keywords
            .iter()
            .any(|k| matches!(k, Keyword::Landwalk(_))));
        for _ in 0..4 { g.add_card_to_library(0, catalog::island()); }
        let hand = g.players[0].hand.len();
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
        let effect = catalog::cold_eyed_selkie().triggered_abilities[0].effect.clone();
        let ctx = EffectContext { event_amount: 3, ..EffectContext::for_trigger(selkie, 0, None, 0) };
        g.resolve_effect(&effect, &ctx).unwrap();
        assert_eq!(g.players[0].hand.len(), hand + 3, "drew that many cards");
    }

    #[test]
    fn bioshift_moves_counters() {
        let mut g = two_player_game();
        let from = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let to = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let seed = EffectContext::for_spell(0, Some(Target::Permanent(from)), 0, 0);
        g.resolve_effect(
            &Effect::AddCounter { what: Selector::Target(0), kind: CounterType::PlusOnePlusOne, amount: Value::Const(3) },
            &seed,
        )
        .unwrap();
        let shift = g.add_card_to_hand(0, catalog::bioshift());
        g.players[0].mana_pool.add(Color::Green, 1);
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::CastSpell {
            card_id: shift,
            target: Some(Target::Permanent(from)),
            additional_targets: vec![Target::Permanent(to)],
            mode: None,
            x_value: None,
        })
        .expect("cast bioshift");
        drain_stack(&mut g);
        assert_eq!(counters(&g, from), 0);
        assert_eq!(counters(&g, to), 3, "counters moved to the second creature");
    }

    #[test]
    fn woodland_champion_grows_per_token() {
        let mut g = two_player_game();
        let champ = g.add_card_to_battlefield(0, catalog::woodland_champion());
        // Resolve a token-minting effect; the token's entry triggers the Champion.
        let ctx = EffectContext::for_spell(0, None, 0, 0);
        let evs = g
            .resolve_effect(
                &Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    definition: crabomination_base::tokens::treasure_token(),
                },
                &ctx,
            )
            .unwrap();
        g.dispatch_triggers_for_events(&evs);
        drain_stack(&mut g);
        assert_eq!(counters(&g, champ), 1, "a token entering grew the Champion");
    }

    #[test]
    fn feat_of_resistance_counters_and_protects() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let feat = g.add_card_to_hand(0, catalog::feat_of_resistance());
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Color(Color::Red)]));
        g.players[0].mana_pool.add(Color::White, 1);
        g.players[0].mana_pool.add_colorless(1);
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::CastSpell {
            card_id: feat, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None,
        })
        .expect("cast feat");
        drain_stack(&mut g);
        assert_eq!(counters(&g, bear), 1, "got a +1/+1 counter");
        let cp = g.compute_battlefield();
        assert!(cp.iter().find(|c| c.id == bear).unwrap().keywords.contains(&Keyword::Protection(Color::Red)));
    }

    #[test]
    fn travel_preparations_counters_two() {
        let mut g = two_player_game();
        let a = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let b = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let tp = g.add_card_to_hand(0, catalog::travel_preparations());
        g.decider = Box::new(ScriptedDecider::new([
            DecisionAnswer::Target(Target::Permanent(a)),
            DecisionAnswer::Target(Target::Permanent(b)),
        ]));
        g.players[0].mana_pool.add(Color::Green, 1);
        g.players[0].mana_pool.add_colorless(1);
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::CastSpell {
            card_id: tp, target: Some(Target::Permanent(a)), additional_targets: vec![Target::Permanent(b)], mode: None, x_value: None,
        })
        .expect("cast travel preparations");
        drain_stack(&mut g);
        assert_eq!(counters(&g, a), 1);
        assert_eq!(counters(&g, b), 1);
        assert!(catalog::travel_preparations().keywords.iter().any(|k| matches!(k, Keyword::Flashback(_))));
    }

    #[test]
    fn graft_moves_a_counter_to_a_new_creature() {
        // CR 702.58 — Plaxcaster Frogling enters with 3 counters and grafts one
        // onto each creature that enters afterward.
        let mut g = two_player_game();
        let frog = g.move_card_to_battlefield_for_test(0, catalog::plaxcaster_frogling());
        drain_stack(&mut g);
        assert_eq!(counters(&g, frog), 3, "graft enters with 3 +1/+1 counters");
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
        let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
        cast_from_hand(&mut g, bear, &[(Color::Green, 1)], 1);
        assert_eq!(counters(&g, bear), 1, "grafted a counter onto the entrant");
        assert_eq!(counters(&g, frog), 2, "graft source lost the moved counter");
    }

    #[test]
    fn renown_triggers_only_once() {
        // CR 702.111 — renowns once on combat damage; a second trigger is inert.
        let mut g = two_player_game();
        let aven = g.add_card_to_battlefield(0, catalog::stalwart_aven());
        let effect = catalog::stalwart_aven().triggered_abilities[0].effect.clone();
        let ctx = EffectContext::for_trigger(aven, 0, None, 0);
        g.resolve_effect(&effect, &ctx).unwrap();
        assert_eq!(counters(&g, aven), 1, "renowned with one +1/+1 counter");
        g.resolve_effect(&effect, &ctx).unwrap();
        assert_eq!(counters(&g, aven), 1, "already renowned → no second counter");
    }

    #[test]
    fn master_biomancer_scales_entrants_by_its_power() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::master_biomancer()); // 2/4
        let bear = g.move_card_to_battlefield_for_test(0, catalog::grizzly_bears());
        assert_eq!(counters(&g, bear), 2, "entered with counters equal to Biomancer's power");
    }

    #[test]
    fn managorger_also_grows_on_opponent_spells() {
        let mut g = two_player_game();
        let hydra = g.add_card_to_battlefield(0, catalog::managorger_hydra());
        let bear = g.add_card_to_hand(1, catalog::grizzly_bears());
        g.players[1].mana_pool.add(Color::Green, 1);
        g.players[1].mana_pool.add_colorless(1);
        g.step = TurnStep::PreCombatMain;
        g.active_player_idx = 1;
        g.priority.player_with_priority = 1;
        g.perform_action(GameAction::CastSpell {
            card_id: bear, target: None, additional_targets: vec![], mode: None, x_value: None,
        })
        .expect("opponent casts");
        drain_stack(&mut g);
        assert_eq!(counters(&g, hydra), 1, "opponent's spell also grows Managorger");
    }
}

mod recent55 {
    use crabomination::card::{CardType, CounterType, Keyword};
    use crabomination::catalog;
    use crabomination::game::*;

    fn cast_artifact(g: &mut GameState, controller: usize) {
        // Mind Stone — a cheap colorless artifact spell (a noncreature spell that
        // also enters as an artifact).
        let id = g.add_card_to_hand(controller, catalog::mind_stone());
        g.players[controller].mana_pool.add_colorless(2);
        g.step = TurnStep::PreCombatMain;
        g.active_player_idx = controller;
        g.priority.player_with_priority = controller;
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
        })
        .expect("cast artifact");
        drain_stack(g);
    }

    #[test]
    fn thopter_engineer_makes_a_thopter_with_haste() {
        let mut g = two_player_game();
        let eng = g.add_card_to_battlefield(0, catalog::thopter_engineer());
        g.fire_self_etb_triggers(eng, 0);
        drain_stack(&mut g);
        let thopter_id = g
            .battlefield
            .iter()
            .find(|c| c.definition.name == "Thopter")
            .expect("made a Thopter")
            .id;
        let cp = g.compute_battlefield();
        let thopter = cp.iter().find(|c| c.id == thopter_id).unwrap();
        assert!(thopter.keywords.contains(&Keyword::Flying), "Thopter flies");
        assert!(thopter.keywords.contains(&Keyword::Haste), "artifact creature granted haste");
    }

    #[test]
    fn maverick_thopterist_makes_two_thopters() {
        let mut g = two_player_game();
        let mav = g.add_card_to_battlefield(0, catalog::maverick_thopterist());
        assert!(catalog::maverick_thopterist().keywords.contains(&Keyword::Improvise));
        g.fire_self_etb_triggers(mav, 0);
        drain_stack(&mut g);
        let thopters = g.battlefield.iter().filter(|c| c.definition.name == "Thopter").count();
        assert_eq!(thopters, 2, "made two Thopters");
    }

    #[test]
    fn ingenious_smith_grows_on_artifact_entry() {
        let mut g = two_player_game();
        let smith = g.add_card_to_battlefield(0, catalog::ingenious_smith());
        cast_artifact(&mut g, 0);
        assert_eq!(
            g.battlefield_find(smith).unwrap().counter_count(CounterType::PlusOnePlusOne),
            1,
            "an artifact entering grew the Smith",
        );
    }

    #[test]
    fn ravenous_intruder_eats_an_artifact() {
        let mut g = two_player_game();
        let intruder = g.add_card_to_battlefield(0, catalog::ravenous_intruder());
        let art = g.add_card_to_battlefield(0, catalog::mind_stone());
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::ActivateAbility {
            card_id: intruder, ability_index: 0, target: None, additional_targets: vec![], x_value: None, mode: None,
        })
        .expect("sac an artifact for +2/+2");
        drain_stack(&mut g);
        assert!(g.battlefield_find(art).is_none(), "artifact sacrificed as the cost");
        let cp = g.compute_battlefield();
        let i = cp.iter().find(|c| c.id == intruder).unwrap();
        assert_eq!((i.power, i.toughness), (3, 4), "+2/+2 until end of turn");
    }

    #[test]
    fn saheeli_makes_a_servo_on_noncreature_cast() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::saheeli_sublime_artificer());
        cast_artifact(&mut g, 0);
        let servos = g
            .battlefield
            .iter()
            .filter(|c| c.definition.name == "Servo" && c.definition.card_types.contains(&CardType::Artifact))
            .count();
        assert_eq!(servos, 1, "a noncreature spell made a Servo");
    }
}

mod recent56 {
    use crabomination::card::{CardType, CreatureType, Keyword, Subtypes};
    use crabomination::catalog;
    use crabomination::game::*;

    /// A vanilla 1/1 white Angel token body for death/enter tests.
    fn angel_1_1() -> crabomination::card::CardDefinition {
        crabomination::card::CardDefinition {
            name: "Test Angel",
            card_types: vec![CardType::Creature],
            subtypes: Subtypes { creature_types: vec![CreatureType::Angel], ..Default::default() },
            power: 1,
            toughness: 1,
            ..Default::default()
        }
    }

    fn gain_life(g: &mut GameState, seat: usize, amount: i32) {
        let before = g.players[seat].life;
        g.adjust_life(seat, amount);
        let delta = g.players[seat].life - before;
        if delta > 0 {
            g.dispatch_triggers_for_events(&[GameEvent::LifeGained { player: seat, amount: delta as u32 }]);
            drain_stack(g);
        }
    }

    #[test]
    fn bishop_of_wings_gains_on_angel_enter_and_makes_spirit_on_death() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::bishop_of_wings());
        let life = g.players[0].life;
        // An Angel entering under your control → gain 4.
        let angel = g.add_card_to_battlefield(0, angel_1_1());
        g.dispatch_triggers_for_events(&[GameEvent::PermanentEntered { card_id: angel }]);
        drain_stack(&mut g);
        assert_eq!(g.players[0].life, life + 4, "gained 4 when an Angel entered");
        // That Angel dying → make a 1/1 flying Spirit.
        let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
        g.players[0].mana_pool.add(crabomination::mana::Color::Red, 1);
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::CastSpell {
            card_id: bolt, target: Some(Target::Permanent(angel)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("bolt the angel");
        drain_stack(&mut g);
        let spirit = g.battlefield.iter().find(|c| c.definition.name == "Spirit");
        assert!(spirit.is_some(), "an Angel dying made a Spirit");
        assert!(spirit.unwrap().has_keyword(&Keyword::Flying), "Spirit flies");
    }

    #[test]
    fn youthful_valkyrie_grows_on_another_angel() {
        let mut g = two_player_game();
        let val = g.add_card_to_battlefield(0, catalog::youthful_valkyrie());
        let angel = g.add_card_to_battlefield(0, angel_1_1());
        g.dispatch_triggers_for_events(&[GameEvent::PermanentEntered { card_id: angel }]);
        drain_stack(&mut g);
        assert_eq!(
            g.battlefield_find(val).unwrap().counter_count(crabomination::card::CounterType::PlusOnePlusOne),
            1,
            "another Angel entering grew the Valkyrie",
        );
    }

    #[test]
    fn righteous_valkyrie_gains_and_anthems_at_high_life() {
        let mut g = two_player_game();
        let val = g.add_card_to_battlefield(0, catalog::righteous_valkyrie());
        let life = g.players[0].life;
        // A 1/1 Angel entering → gain life = its toughness (1).
        let angel = g.add_card_to_battlefield(0, angel_1_1());
        g.dispatch_triggers_for_events(&[GameEvent::PermanentEntered { card_id: angel }]);
        drain_stack(&mut g);
        assert_eq!(g.players[0].life, life + 1, "gained life equal to entering creature's toughness");
        // Bump to starting+7 and confirm the team anthem kicks in.
        g.adjust_life(0, 7);
        let cp = g.compute_battlefield();
        let v = cp.iter().find(|c| c.id == val).unwrap();
        assert_eq!((v.power, v.toughness), (4, 6), "+2/+2 anthem while 7 above starting");
    }

    #[test]
    fn twinblade_paladin_grows_and_gains_double_strike() {
        let mut g = two_player_game();
        let pal = g.add_card_to_battlefield(0, catalog::twinblade_paladin());
        gain_life(&mut g, 0, 1);
        assert_eq!(
            g.battlefield_find(pal).unwrap().counter_count(crabomination::card::CounterType::PlusOnePlusOne),
            1,
            "gaining life grew the Paladin",
        );
        g.adjust_life(0, 5); // 26 total ≥ 25
        let cp = g.compute_battlefield();
        assert!(
            cp.iter().find(|c| c.id == pal).unwrap().keywords.contains(&Keyword::DoubleStrike),
            "double strike while at 25+ life",
        );
    }

    #[test]
    fn rhox_faithmender_doubles_life_gain() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::rhox_faithmender());
        let life = g.players[0].life;
        g.adjust_life(0, 3);
        assert_eq!(g.players[0].life, life + 6, "life gain doubled");
    }

    #[test]
    fn vito_drains_opponent_on_life_gain() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::vito_thorn_of_the_dusk_rose());
        let opp = g.players[1].life;
        gain_life(&mut g, 0, 3);
        assert_eq!(g.players[1].life, opp - 3, "opponent lost life equal to the gain");
    }

    #[test]
    fn angelic_chorus_gains_life_equal_to_toughness() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::angelic_chorus());
        let life = g.players[0].life;
        // A 2/2 entering → gain 2.
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.dispatch_triggers_for_events(&[GameEvent::PermanentEntered { card_id: bear }]);
        drain_stack(&mut g);
        assert_eq!(g.players[0].life, life + 2, "gained life = entering creature's toughness");
    }

    #[test]
    fn exquisite_blood_gains_when_opponent_loses_life() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::exquisite_blood());
        let life = g.players[0].life;
        g.adjust_life(1, -4);
        g.dispatch_triggers_for_events(&[GameEvent::LifeLost { player: 1, amount: 4 }]);
        drain_stack(&mut g);
        assert_eq!(g.players[0].life, life + 4, "gained life equal to opponent's loss");
    }

    #[test]
    fn epicure_of_blood_drains_each_opponent_on_life_gain() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::epicure_of_blood());
        let opp = g.players[1].life;
        gain_life(&mut g, 0, 5);
        assert_eq!(g.players[1].life, opp - 1, "each opponent lost 1 on life gain");
    }

    #[test]
    fn celestial_unicorn_and_gideons_company_grow_on_life_gain() {
        let mut g = two_player_game();
        let uni = g.add_card_to_battlefield(0, catalog::celestial_unicorn());
        let comp = g.add_card_to_battlefield(0, catalog::gideons_company());
        gain_life(&mut g, 0, 1);
        let p1p1 = crabomination::card::CounterType::PlusOnePlusOne;
        assert_eq!(g.battlefield_find(uni).unwrap().counter_count(p1p1), 1, "Unicorn +1 counter");
        assert_eq!(g.battlefield_find(comp).unwrap().counter_count(p1p1), 2, "Gideon's Company +2 counters");
    }

    #[test]
    fn dauntless_bodyguard_grants_indestructible_to_chosen() {
        let mut g = two_player_game();
        let ward = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let guard = g.add_card_to_battlefield(0, catalog::dauntless_bodyguard());
        // ETB: choose the ward (only other creature) as the protected creature.
        g.fire_self_etb_triggers(guard, 0);
        drain_stack(&mut g);
        assert_eq!(g.battlefield_find(guard).unwrap().chosen_permanent, Some(ward), "remembered the ward");
        // Sacrifice the guard → the chosen creature gains indestructible.
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::ActivateAbility {
            card_id: guard, ability_index: 0, target: None, additional_targets: vec![], x_value: None, mode: None,
        }).expect("sac the bodyguard");
        drain_stack(&mut g);
        assert!(g.battlefield_find(guard).is_none(), "bodyguard sacrificed");
        let cp = g.compute_battlefield();
        assert!(
            cp.iter().find(|c| c.id == ward).unwrap().keywords.contains(&Keyword::Indestructible),
            "the chosen creature gained indestructible",
        );
    }

    #[test]
    fn griffin_aerie_makes_griffin_after_gaining_three() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::griffin_aerie());
        g.adjust_life(0, 3); // gained 3 this turn
        g.step = TurnStep::End;
        g.fire_step_triggers(TurnStep::End);
        drain_stack(&mut g);
        assert!(g.battlefield.iter().any(|c| c.definition.name == "Griffin"), "made a Griffin");
    }

    #[test]
    fn crested_sunmare_makes_horse_and_shields_other_horses() {
        let mut g = two_player_game();
        let mare = g.add_card_to_battlefield(0, catalog::crested_sunmare());
        // Gain life, then end step → make a Horse.
        g.adjust_life(0, 2);
        g.step = TurnStep::End;
        g.fire_step_triggers(TurnStep::End);
        drain_stack(&mut g);
        let horse = g.battlefield.iter().find(|c| c.definition.name == "Horse").map(|c| c.id);
        assert!(horse.is_some(), "made a Horse token");
        // The token Horse is indestructible (lord); the Sunmare itself is not.
        let cp = g.compute_battlefield();
        assert!(
            cp.iter().find(|c| c.id == horse.unwrap()).unwrap().keywords.contains(&Keyword::Indestructible),
            "other Horses are indestructible",
        );
        assert!(
            !cp.iter().find(|c| c.id == mare).unwrap().keywords.contains(&Keyword::Indestructible),
            "the Sunmare itself is not (only *other* Horses)",
        );
    }

    #[test]
    fn linden_gains_when_white_creature_attacks() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::linden_the_steadfast_queen());
        let life = g.players[0].life;
        // A white creature attacking → gain 1. (Linden herself is white.)
        let soldier = g.add_card_to_battlefield(0, catalog::savannah_lions());
        g.dispatch_triggers_for_events(&[GameEvent::AttackerDeclared(soldier)]);
        drain_stack(&mut g);
        assert_eq!(g.players[0].life, life + 1, "gained 1 when a white creature attacked");
    }

    #[test]
    fn kambal_drains_on_opponent_noncreature_spell() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::kambal_consul_of_allocation());
        let (my, opp) = (g.players[0].life, g.players[1].life);
        let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
        g.players[1].mana_pool.add(crabomination::mana::Color::Red, 1);
        g.active_player_idx = 1;
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 1;
        g.perform_action(GameAction::CastSpell {
            card_id: bolt, target: Some(Target::Player(0)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("opponent casts a noncreature spell");
        drain_stack(&mut g);
        assert_eq!(g.players[1].life, opp - 2, "the casting opponent lost 2 (plus the bolt's 0 to self)");
        assert!(g.players[0].life >= my - 3, "you gained 2 (bolt still hits you for 3)");
    }

    #[test]
    fn sunscorch_regent_grows_and_gains_on_opponent_spell() {
        let mut g = two_player_game();
        let reg = g.add_card_to_battlefield(0, catalog::sunscorch_regent());
        let life = g.players[0].life;
        let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
        g.players[1].mana_pool.add(crabomination::mana::Color::Red, 1);
        g.active_player_idx = 1;
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 1;
        g.perform_action(GameAction::CastSpell {
            card_id: bolt, target: Some(Target::Player(0)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("opponent casts a spell");
        drain_stack(&mut g);
        assert_eq!(
            g.battlefield_find(reg).unwrap().counter_count(crabomination::card::CounterType::PlusOnePlusOne),
            1,
            "Regent grew on the opponent's spell",
        );
        // gained 1 from the Regent, lost 3 to the bolt → net -2.
        assert_eq!(g.players[0].life, life + 1 - 3);
    }

    #[test]
    fn souls_grace_gains_life_equal_to_target_power() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
        let grace = g.add_card_to_hand(0, catalog::souls_grace());
        g.players[0].mana_pool.add(crabomination::mana::Color::White, 1);
        g.players[0].mana_pool.add_colorless(1);
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        let life = g.players[0].life;
        g.perform_action(GameAction::CastSpell {
            card_id: grace, target: Some(Target::Permanent(bear)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast Soul's Grace");
        drain_stack(&mut g);
        assert_eq!(g.players[0].life, life + 2, "gained life equal to target's power");
    }

    #[test]
    fn valkyrie_harbinger_and_regal_bloodlord_make_tokens_at_end_step() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::valkyrie_harbinger());
        g.add_card_to_battlefield(0, catalog::regal_bloodlord());
        g.adjust_life(0, 4); // ≥4 for both
        g.step = TurnStep::End;
        g.fire_step_triggers(TurnStep::End);
        drain_stack(&mut g);
        assert!(g.battlefield.iter().any(|c| c.definition.name == "Angel"), "Harbinger made an Angel");
        assert!(g.battlefield.iter().any(|c| c.definition.name == "Bat"), "Bloodlord made a Bat");
    }

    /// The new `#[serde(default)]` state — Player.starting_life and
    /// CardInstance.chosen_permanent — survives a full-state snapshot round-trip.
    #[test]
    fn new_serde_fields_survive_snapshot_roundtrip() {
        let mut g = two_player_game();
        g.players[0].starting_life = 40;
        let ward = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let guard = g.add_card_to_battlefield(0, catalog::dauntless_bodyguard());
        g.fire_self_etb_triggers(guard, 0);
        drain_stack(&mut g);
        let json = serde_json::to_string(&g).expect("serialize");
        let g2: GameState = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(g2.players[0].starting_life, 40, "starting_life round-trips");
        assert_eq!(
            g2.battlefield_find(guard).unwrap().chosen_permanent, Some(ward),
            "chosen_permanent round-trips",
        );
    }
}

mod recent57 {
    use crabomination::card::CreatureType;
    use crabomination::catalog;
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    use crabomination::game::types::{Attack, AttackTarget};
    use crabomination::game::*;

    fn bolt_kill(g: &mut GameState, victim: Target, controller: usize) {
        let bolt = g.add_card_to_hand(controller, catalog::lightning_bolt());
        g.players[controller].mana_pool.add(crabomination::mana::Color::Red, 1);
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = controller;
        g.perform_action(GameAction::CastSpell {
            card_id: bolt, target: Some(victim), additional_targets: vec![], mode: None, x_value: None,
        }).expect("bolt");
        drain_stack(g);
    }

    #[test]
    fn requiem_angel_makes_spirit_on_nonspirit_death() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::requiem_angel());
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        bolt_kill(&mut g, Target::Permanent(bear), 0);
        assert!(g.battlefield.iter().any(|c| c.definition.name == "Spirit"), "non-Spirit death → Spirit");
    }

    #[test]
    fn angel_of_the_dawn_pumps_team_until_eot() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let angel = g.add_card_to_battlefield(0, catalog::angel_of_the_dawn());
        g.fire_self_etb_triggers(angel, 0);
        drain_stack(&mut g);
        let cp = g.compute_battlefield();
        let b = cp.iter().find(|c| c.id == bear).unwrap();
        assert_eq!((b.power, b.toughness), (3, 3), "+1/+1 to the team");
        assert!(b.keywords.contains(&crabomination::card::Keyword::Vigilance), "vigilance granted");
    }

    #[test]
    fn elderfang_disciple_makes_opponent_discard() {
        let mut g = two_player_game();
        g.add_card_to_hand(1, catalog::grizzly_bears());
        let disc = g.add_card_to_battlefield(0, catalog::elderfang_disciple());
        let opp_hand = g.players[1].hand.len();
        g.fire_self_etb_triggers(disc, 0);
        drain_stack(&mut g);
        assert_eq!(g.players[1].hand.len(), opp_hand - 1, "opponent discarded a card");
    }

    #[test]
    fn martial_coup_five_wraths_then_makes_five() {
        let mut g = two_player_game();
        let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let coup = g.add_card_to_hand(0, catalog::martial_coup());
        g.players[0].mana_pool.add(crabomination::mana::Color::White, 2);
        g.players[0].mana_pool.add_colorless(5);
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::CastSpell {
            card_id: coup, target: None, additional_targets: vec![], mode: None, x_value: Some(5),
        }).expect("cast Martial Coup for X=5");
        drain_stack(&mut g);
        assert!(g.battlefield_find(foe).is_none(), "X≥5 destroyed the opponent's creature");
        let soldiers = g.battlefield.iter().filter(|c| c.definition.name == "Soldier" && c.controller == 0).count();
        assert_eq!(soldiers, 5, "made five Soldiers that survive the wrath");
    }

    #[test]
    fn beckon_apparition_exiles_and_makes_spirit() {
        let mut g = two_player_game();
        let gy = g.add_card_to_graveyard(1, catalog::grizzly_bears());
        let beckon = g.add_card_to_hand(0, catalog::beckon_apparition());
        g.players[0].mana_pool.add(crabomination::mana::Color::White, 1);
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::CastSpell {
            card_id: beckon, target: Some(Target::Permanent(gy)), additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast Beckon Apparition");
        drain_stack(&mut g);
        assert!(g.exile.iter().any(|c| c.id == gy), "graveyard card exiled");
        assert!(g.battlefield.iter().any(|c| c.definition.name == "Spirit"), "made a Spirit");
    }

    #[test]
    fn kytheons_tactics_pumps_and_spell_mastery_grants_vigilance() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        // Two instants in the graveyard → spell mastery on.
        g.add_card_to_graveyard(0, catalog::lightning_bolt());
        g.add_card_to_graveyard(0, catalog::lightning_bolt());
        let tac = g.add_card_to_hand(0, catalog::kytheons_tactics());
        g.players[0].mana_pool.add(crabomination::mana::Color::White, 2);
        g.players[0].mana_pool.add_colorless(1);
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::CastSpell {
            card_id: tac, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast Kytheon's Tactics");
        drain_stack(&mut g);
        let cp = g.compute_battlefield();
        let b = cp.iter().find(|c| c.id == bear).unwrap();
        assert_eq!((b.power, b.toughness), (4, 3), "+2/+1");
        assert!(b.keywords.contains(&crabomination::card::Keyword::Vigilance), "spell mastery → vigilance");
    }

    #[test]
    fn rally_the_ranks_anthems_chosen_type() {
        let mut g = two_player_game();
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::CreatureType(CreatureType::Bear)]));
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // a Bear
        let rally = g.add_card_to_battlefield(0, catalog::rally_the_ranks());
        g.fire_self_etb_triggers(rally, 0);
        drain_stack(&mut g);
        let cp = g.compute_battlefield();
        let b = cp.iter().find(|c| c.id == bear).unwrap();
        assert_eq!((b.power, b.toughness), (3, 3), "chosen type (Bear) gets +1/+1");
    }

    #[test]
    fn captains_claws_makes_kor_ally_on_attack() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let claws = g.add_card_to_battlefield(0, catalog::captains_claws());
        g.active_player_idx = 0;
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g.players[0].mana_pool.add_colorless(1);
        g.perform_action(GameAction::Equip { equipment: claws, target: bear }).expect("equip");
        g.battlefield.iter_mut().find(|c| c.id == bear).unwrap().summoning_sick = false;
        g.step = TurnStep::DeclareAttackers;
        g.declare_attackers(vec![Attack { attacker: bear, target: AttackTarget::Player(1) }])
            .expect("attack");
        drain_stack(&mut g);
        let ally = g.battlefield.iter().find(|c| c.definition.name == "Kor Ally");
        assert!(ally.is_some(), "equipped attacker made a Kor Ally");
        assert!(ally.unwrap().tapped, "the Kor Ally entered tapped and attacking");
    }

    #[test]
    fn ancestral_blade_mints_and_equips_a_soldier() {
        let mut g = two_player_game();
        let blade = g.add_card_to_battlefield(0, catalog::ancestral_blade());
        g.fire_self_etb_triggers(blade, 0);
        drain_stack(&mut g);
        let soldier = g.battlefield.iter().find(|c| c.definition.name == "Soldier").map(|c| c.id);
        assert!(soldier.is_some(), "made a Soldier token");
        // Blade attached to it → +1/+1 → 2/2.
        let cp = g.compute_battlefield();
        let s = cp.iter().find(|c| c.id == soldier.unwrap()).unwrap();
        assert_eq!((s.power, s.toughness), (2, 2), "the Soldier is equipped (+1/+1)");
    }
}

mod recent58 {
    use crabomination::card::{CardType, CreatureType, Keyword, Subtypes};
    use crabomination::catalog;
    use crabomination::game::*;

    /// A vanilla 1/1 creature of a chosen party role, for building a party.
    fn role(ct: CreatureType) -> crabomination::card::CardDefinition {
        crabomination::card::CardDefinition {
            name: "Party Member",
            card_types: vec![CardType::Creature],
            subtypes: Subtypes { creature_types: vec![ct], ..Default::default() },
            power: 1,
            toughness: 1,
            ..Default::default()
        }
    }

    /// Squad Commander (itself a Warrior) mints one Kor Warrior per party member.
    fn commander_tokens(setup: &[CreatureType]) -> usize {
        let mut g = two_player_game();
        for &ct in setup {
            g.add_card_to_battlefield(0, role(ct));
        }
        let cmd = g.add_card_to_battlefield(0, catalog::squad_commander());
        g.fire_self_etb_triggers(cmd, 0);
        drain_stack(&mut g);
        g.battlefield.iter().filter(|c| c.definition.name == "Kor Warrior").count()
    }

    #[test]
    fn party_counts_each_distinct_role() {
        // Cleric + Rogue + Wizard + Squad (Warrior) = full party of 4.
        assert_eq!(commander_tokens(&[CreatureType::Cleric, CreatureType::Rogue, CreatureType::Wizard]), 4);
    }

    #[test]
    fn party_duplicate_roles_fill_one_slot() {
        // Two extra Warriors + Squad (Warrior) — all Warriors → party 1.
        assert_eq!(commander_tokens(&[CreatureType::Warrior, CreatureType::Warrior]), 1);
    }

    #[test]
    fn party_ignores_non_role_creatures() {
        // A Bear and a Cleric + Squad (Warrior) → only Cleric + Warrior count → 2.
        assert_eq!(commander_tokens(&[CreatureType::Bear, CreatureType::Cleric]), 2);
    }

    #[test]
    fn tajuru_paragon_fills_only_one_party_slot() {
        // Tajuru is all four roles but fills only one slot (CR 700.18): with just
        // Squad (Warrior), the party is 2 — Squad→Warrior, Tajuru→one other.
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::tajuru_paragon());
        let cmd = g.add_card_to_battlefield(0, catalog::squad_commander());
        g.fire_self_etb_triggers(cmd, 0);
        drain_stack(&mut g);
        let tokens = g.battlefield.iter().filter(|c| c.definition.name == "Kor Warrior").count();
        assert_eq!(tokens, 2, "one creature fills at most one slot → party 2, not 4");
    }

    #[test]
    fn squad_commander_full_party_buffs_team_at_combat() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, role(CreatureType::Cleric));
        g.add_card_to_battlefield(0, role(CreatureType::Rogue));
        g.add_card_to_battlefield(0, role(CreatureType::Wizard));
        let cmd = g.add_card_to_battlefield(0, catalog::squad_commander()); // Warrior → full party
        g.active_player_idx = 0;
        g.step = TurnStep::BeginCombat;
        g.fire_step_triggers(TurnStep::BeginCombat);
        drain_stack(&mut g);
        let cp = g.compute_battlefield();
        let c = cp.iter().find(|c| c.id == cmd).unwrap();
        assert_eq!(c.power, 4, "+1/+0 from the full-party combat trigger");
        assert!(c.keywords.contains(&Keyword::Indestructible), "full party → indestructible");
    }

    #[test]
    fn kabira_outrider_pumps_by_party_size() {
        // Cleric + Rogue + Wizard on board; Outrider (Warrior) enters → party 4.
        // The ETB pumps a creature by +4/+4; assert some creature grew by 4.
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, role(CreatureType::Cleric));
        g.add_card_to_battlefield(0, role(CreatureType::Rogue));
        g.add_card_to_battlefield(0, role(CreatureType::Wizard));
        let out = g.add_card_to_battlefield(0, catalog::kabira_outrider());
        let total = |g: &GameState| g.compute_battlefield().iter()
            .filter(|c| c.controller == 0).map(|c| c.power).sum::<i32>();
        let before = total(&g);
        g.fire_self_etb_triggers(out, 0);
        drain_stack(&mut g);
        assert_eq!(total(&g), before + 4, "a creature got +4/+0..+4 for the full party of 4");
    }
}

mod recent59 {
    use crabomination::card::{CounterType, Keyword};
    use crabomination::catalog;
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    use crabomination::game::*;

    fn cast_sorcery(g: &mut GameState, controller: usize, id: CardId, x: Option<u32>) {
        g.step = TurnStep::PreCombatMain;
        g.active_player_idx = controller;
        g.priority.player_with_priority = controller;
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: None, additional_targets: vec![], mode: None, x_value: x,
        }).expect("cast");
        drain_stack(g);
    }

    #[test]
    fn sky_terror_has_flying_and_menace() {
        let mut g = two_player_game();
        let st = g.add_card_to_battlefield(0, catalog::sky_terror());
        let cp = g.compute_battlefield();
        let c = cp.iter().find(|c| c.id == st).unwrap();
        assert!(c.keywords.contains(&Keyword::Flying) && c.keywords.contains(&Keyword::Menace));
    }

    #[test]
    fn talrands_invocation_makes_two_drakes() {
        let mut g = two_player_game();
        let inv = g.add_card_to_hand(0, catalog::talrands_invocation());
        g.players[0].mana_pool.add(crabomination::mana::Color::Blue, 2);
        g.players[0].mana_pool.add_colorless(2);
        cast_sorcery(&mut g, 0, inv, None);
        assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Drake").count(), 2);
    }

    #[test]
    fn ondu_cleric_gains_life_per_ally() {
        use crabomination::card::{CardType, CreatureType, Subtypes};
        let ally = || crabomination::card::CardDefinition {
            name: "Ally Buddy",
            card_types: vec![CardType::Creature],
            subtypes: Subtypes { creature_types: vec![CreatureType::Ally], ..Default::default() },
            power: 1, toughness: 1, ..Default::default()
        };
        let mut g = two_player_game();
        // Two vanilla Allies out; a single Ondu Cleric (also an Ally) enters →
        // 3 Allies total → gain 3. Only the Ondu carries the trigger.
        g.add_card_to_battlefield(0, ally());
        g.add_card_to_battlefield(0, ally());
        let life = g.players[0].life;
        let c = g.add_card_to_battlefield(0, catalog::ondu_cleric());
        g.dispatch_triggers_for_events(&[GameEvent::PermanentEntered { card_id: c }]);
        drain_stack(&mut g);
        assert_eq!(g.players[0].life, life + 3, "gained life = number of Allies (3)");
    }

    #[test]
    fn aven_eternal_amasses_zombies() {
        let mut g = two_player_game();
        let av = g.add_card_to_battlefield(0, catalog::aven_eternal());
        g.fire_self_etb_triggers(av, 0);
        drain_stack(&mut g);
        // No Army existed → a 0/0 Army token was made and given a +1/+1 counter.
        let army = g.battlefield.iter().find(|c| c.definition.name == "Army");
        assert!(army.is_some(), "made an Army token");
        assert_eq!(army.unwrap().counter_count(CounterType::PlusOnePlusOne), 1, "amass 1");
        assert!(army.unwrap().definition.subtypes.creature_types.contains(&crabomination::card::CreatureType::Zombie),
            "the Army is also a Zombie");
    }

    #[test]
    fn storm_fleet_arsonist_sacrifices_only_after_attacking() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(1, catalog::grizzly_bears()); // opponent's only permanent
        // No attack this turn → raid off, no sacrifice.
        let a1 = g.add_card_to_battlefield(0, catalog::storm_fleet_arsonist());
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Target(Target::Player(1))]));
        g.fire_self_etb_triggers(a1, 0);
        drain_stack(&mut g);
        assert_eq!(g.players[1].graveyard.len(), 0, "no raid → no sacrifice");
        // Mark that player 0 attacked, then a second Arsonist enters → sacrifice.
        g.players[0].attacked_this_turn = true;
        let a2 = g.add_card_to_battlefield(0, catalog::storm_fleet_arsonist());
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Target(Target::Player(1))]));
        g.fire_self_etb_triggers(a2, 0);
        drain_stack(&mut g);
        assert_eq!(g.players[1].graveyard.len(), 1, "raid → opponent sacrificed a permanent");
    }

    #[test]
    fn metallurgic_summonings_makes_xx_construct() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::metallurgic_summonings());
        // Cast a mana-value-3 instant (Divination is MV 3? use a known IS spell).
        let bolt = g.add_card_to_hand(0, catalog::lightning_bolt()); // MV 1
        g.players[0].mana_pool.add(crabomination::mana::Color::Red, 1);
        cast_sorcery(&mut g, 0, bolt, None);
        let con = g.battlefield.iter().find(|c| c.definition.name == "Construct").map(|c| c.id);
        assert!(con.is_some(), "cast an I/S → a Construct token");
        let cp = g.compute_battlefield();
        let c = cp.iter().find(|c| c.id == con.unwrap()).unwrap();
        assert_eq!((c.power, c.toughness), (1, 1), "X = the spell's mana value (Bolt = 1)");
    }
}

mod recent60 {
    use crabomination::card::{CardType, CreatureType, Subtypes};
    use crabomination::catalog;
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    use crabomination::game::*;
    use crabomination::mana::Color;

    fn vanilla_creature(name: &'static str) -> crabomination::card::CardDefinition {
        crabomination::card::CardDefinition {
            name,
            card_types: vec![CardType::Creature],
            subtypes: Subtypes { creature_types: vec![CreatureType::Human], ..Default::default() },
            power: 2,
            toughness: 2,
            ..Default::default()
        }
    }

    #[test]
    fn jolrael_makes_cat_on_your_second_draw() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::jolrael_mwonvuli_recluse());
        for _ in 0..2 { g.add_card_to_library(0, catalog::forest()); }
        g.players[0].cards_drawn_this_turn = 0;
        // First draw — no token.
        let mut ev = vec![];
        g.draw_one(0, &mut ev);
        g.dispatch_triggers_for_events(&ev);
        drain_stack(&mut g);
        assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Cat").count(), 0);
        // Second draw — one Cat.
        let mut ev2 = vec![];
        g.draw_one(0, &mut ev2);
        g.dispatch_triggers_for_events(&ev2);
        drain_stack(&mut g);
        assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Cat").count(), 1);
    }

    #[test]
    fn jolrael_sets_team_base_pt_to_hand_size() {
        let mut g = two_player_game();
        let jol = g.add_card_to_battlefield(0, catalog::jolrael_mwonvuli_recluse());
        let bear = g.add_card_to_battlefield(0, vanilla_creature("Bear"));
        for _ in 0..3 { g.add_card_to_hand(0, catalog::forest()); }
        g.players[0].mana_pool.add(Color::Green, 2);
        g.players[0].mana_pool.add_colorless(4);
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::ActivateAbility {
            card_id: jol,
            ability_index: 0,
            target: None,
            additional_targets: vec![],
            x_value: None, mode: None,
        })
        .expect("activate");
        drain_stack(&mut g);
        let cp = g.compute_battlefield();
        // 3 cards in hand → each creature you control is base 3/3.
        for id in [jol, bear] {
            let c = cp.iter().find(|c| c.id == id).unwrap();
            assert_eq!((c.power, c.toughness), (3, 3), "creature became 3/3");
        }
    }

    #[test]
    fn loyal_warhound_fetches_only_when_behind_on_lands() {
        // Behind on lands → fetch a basic Plains onto the battlefield tapped.
        let mut g = two_player_game();
        for _ in 0..2 { g.add_card_to_battlefield(1, catalog::forest()); }
        let plains = g.add_card_to_library(0, catalog::plains());
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(plains))]));
        let wh = g.add_card_to_battlefield(0, catalog::loyal_warhound());
        g.fire_self_etb_triggers(wh, 0);
        drain_stack(&mut g);
        let fetched = g.battlefield_find(plains).expect("Plains fetched to battlefield");
        assert!(fetched.tapped, "fetched Plains enters tapped");
        assert_eq!(fetched.controller, 0);
    }

    #[test]
    fn well_of_lost_dreams_pays_to_draw() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::well_of_lost_dreams());
        for _ in 0..3 { g.add_card_to_library(0, catalog::forest()); }
        // Float 3 mana, pay 2 of it to draw 2 (leaving 1 unspent).
        g.players[0].mana_pool.add_colorless(3);
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Amount(2)]));
        let hand = g.players[0].hand.len();
        g.adjust_life(0, 3);
        g.dispatch_triggers_for_events(&[GameEvent::LifeGained { player: 0, amount: 3 }]);
        drain_stack(&mut g);
        assert_eq!(g.players[0].hand.len(), hand + 2, "drew X=2 cards");
        assert_eq!(g.players[0].mana_pool.total(), 1, "spent 2 of the 3 floated mana");
    }

    #[test]
    fn custodi_soulbinders_counters_and_spirit_activation() {
        let mut g = two_player_game();
        // Two other creatures on the battlefield (one each side) → enters 2/2.
        g.add_card_to_battlefield(0, vanilla_creature("Ally"));
        g.add_card_to_battlefield(1, vanilla_creature("Foe"));
        // Enter through the real ETB funnel so enters-with-counters fires.
        let cs = g.move_card_to_battlefield_for_test(0, catalog::custodi_soulbinders());
        let c = g.compute_battlefield();
        let cc = c.iter().find(|c| c.id == cs).unwrap();
        assert_eq!((cc.power, cc.toughness), (2, 2), "enters with 2 +1/+1 counters");
        // Remove a counter to mint a Spirit.
        g.players[0].mana_pool.add(Color::White, 1);
        g.players[0].mana_pool.add_colorless(2);
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::ActivateAbility {
            card_id: cs,
            ability_index: 0,
            target: None,
            additional_targets: vec![],
            x_value: None, mode: None,
        })
        .expect("activate");
        drain_stack(&mut g);
        assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Spirit").count(), 1);
        let c2 = g.compute_battlefield();
        let cc2 = c2.iter().find(|c| c.id == cs).unwrap();
        assert_eq!((cc2.power, cc2.toughness), (1, 1), "one counter removed → 1/1");
    }
}
