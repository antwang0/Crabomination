//! Tests for recentN card batches 81-90 (merged from per-batch micro-files).

mod recent81 {
    use crabomination::card::{CounterType, Keyword};
    use crabomination::catalog;
    use crabomination::game::two_player_game;
    use crabomination::game::types::{Attack, AttackTarget, Target, TurnStep};
    use crabomination::game::*;

    fn advance_to(g: &mut GameState, step: TurnStep) {
        while g.step != step {
            g.perform_action(GameAction::PassPriority).expect("pass priority");
        }
    }

    #[test]
    fn vampiric_rites_sacrifices_for_life_and_a_card() {
        let mut g = two_player_game();
        let rites = g.add_card_to_battlefield(0, catalog::vampiric_rites());
        g.add_card_to_battlefield(0, catalog::grizzly_bears());
        for _ in 0..3 { g.add_card_to_library(0, catalog::plains()); }
        let life = g.players[0].life;
        let hand = g.players[0].hand.len();
        g.players[0].mana_pool.add(crabomination::mana::Color::Black, 1);
        g.players[0].mana_pool.add_colorless(1);
        g.perform_action(GameAction::ActivateAbility {
            card_id: rites, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
        }).expect("activate Vampiric Rites");
        drain_stack(&mut g);
        assert_eq!(g.players[0].life, life + 1, "gained 1 life");
        assert_eq!(g.players[0].hand.len(), hand + 1, "drew a card");
        assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Grizzly Bears").count(), 0,
            "the creature was sacrificed");
    }

    #[test]
    fn blasting_station_sacrifices_to_ping() {
        let mut g = two_player_game();
        let station = g.add_card_to_battlefield(0, catalog::blasting_station());
        g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let life = g.players[1].life;
        g.perform_action(GameAction::ActivateAbility {
            card_id: station, ability_index: 0, target: Some(Target::Player(1)),
            additional_targets: vec![], x_value: None,
        }).expect("activate Blasting Station");
        drain_stack(&mut g);
        assert_eq!(g.players[1].life, life - 1, "1 damage to opponent");
    }

    #[test]
    fn seize_the_spoils_discards_for_two_cards_and_a_treasure() {
        let mut g = two_player_game();
        for _ in 0..4 { g.add_card_to_library(0, catalog::plains()); }
        let discard_fodder = g.add_card_to_hand(0, catalog::plains());
        let id = g.add_card_to_hand(0, catalog::seize_the_spoils());
        let hand = g.players[0].hand.len();
        g.players[0].mana_pool.add(crabomination::mana::Color::Red, 1);
        g.players[0].mana_pool.add_colorless(2);
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast Seize the Spoils");
        drain_stack(&mut g);
        // -Seize -discard +2 drawn = net -0 from the initial `hand` (which counted Seize + fodder).
        assert_eq!(g.players[0].hand.len(), hand - 2 + 2, "discarded one, drew two");
        let _ = discard_fodder;
        assert!(g.battlefield.iter().any(|c| c.definition.name == "Treasure" && c.controller == 0),
            "created a Treasure");
    }

    #[test]
    fn blood_divination_sacrifices_a_creature_for_three_cards() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::grizzly_bears());
        for _ in 0..3 { g.add_card_to_library(0, catalog::plains()); }
        let id = g.add_card_to_hand(0, catalog::blood_divination());
        let hand = g.players[0].hand.len();
        g.players[0].mana_pool.add(crabomination::mana::Color::Black, 1);
        g.players[0].mana_pool.add_colorless(3);
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast Blood Divination");
        drain_stack(&mut g);
        assert_eq!(g.players[0].hand.len(), hand - 1 + 3, "cast it, drew three");
        assert_eq!(g.battlefield.iter().filter(|c| c.definition.is_creature()).count(), 0,
            "the creature was sacrificed as an additional cost");
    }

    #[test]
    fn snake_umbra_pumps_and_draws_on_combat_damage() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let umbra = g.add_card_to_hand(0, catalog::snake_umbra());
        for _ in 0..3 { g.add_card_to_library(0, catalog::plains()); }
        g.players[0].mana_pool.add(crabomination::mana::Color::Green, 1);
        g.players[0].mana_pool.add_colorless(2);
        cast_at(&mut g, umbra, Target::Permanent(bear));
        // +1/+1 → 3/3.
        assert_eq!(g.computed_permanent(bear).unwrap().power, 3, "Snake Umbra grants +1/+1");
        let hand = g.players[0].hand.len();
        g.clear_sickness(bear);
        advance_to(&mut g, TurnStep::DeclareAttackers);
        g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker: bear, target: AttackTarget::Player(1),
        }])).expect("attack");
        drain_stack(&mut g);
        advance_to(&mut g, TurnStep::CombatDamage);
        drain_stack(&mut g);
        assert_eq!(g.players[0].hand.len(), hand + 1, "drew from combat damage");
    }

    #[test]
    fn curious_obsession_sacrifices_when_you_didnt_attack() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let aura = g.add_card_to_hand(0, catalog::curious_obsession());
        g.players[0].mana_pool.add(crabomination::mana::Color::Blue, 1);
        cast_at(&mut g, aura, Target::Permanent(bear));
        assert!(g.battlefield.iter().any(|c| c.id == aura), "aura attached");
        // End step, no attack this turn → the Aura sacrifices itself.
        g.active_player_idx = 0;
        g.step = TurnStep::End;
        g.fire_step_triggers(TurnStep::End);
        drain_stack(&mut g);
        assert!(!g.battlefield.iter().any(|c| c.id == aura), "Aura sacrificed (didn't attack)");
    }

    #[test]
    fn ageless_entity_grows_on_lifegain() {
        let mut g = two_player_game();
        let ae = g.add_card_to_battlefield(0, catalog::ageless_entity());
        let bless = g.add_card_to_hand(0, catalog::chaplains_blessing());
        g.players[0].mana_pool.add(crabomination::mana::Color::White, 1);
        cast(&mut g, bless); // gain 5
        let counters = g.battlefield_find(ae).unwrap().counter_count(CounterType::PlusOnePlusOne);
        assert_eq!(counters, 5, "5 +1/+1 counters from gaining 5 life");
    }

    #[test]
    fn sunbond_grants_lifegain_growth() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let sunbond = g.add_card_to_hand(0, catalog::sunbond());
        let bless = g.add_card_to_hand(0, catalog::chaplains_blessing());
        g.players[0].mana_pool.add(crabomination::mana::Color::White, 2);
        g.players[0].mana_pool.add_colorless(3);
        cast_at(&mut g, sunbond, Target::Permanent(bear));
        cast(&mut g, bless); // gain 5
        let counters = g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne);
        assert_eq!(counters, 5, "enchanted creature grew by 5");
    }

    #[test]
    fn nyx_fleece_ram_gains_life_each_upkeep() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::nyx_fleece_ram());
        let life = g.players[0].life;
        g.active_player_idx = 0;
        g.step = TurnStep::Upkeep;
        g.fire_step_triggers(TurnStep::Upkeep);
        drain_stack(&mut g);
        assert_eq!(g.players[0].life, life + 1, "gained 1 at upkeep");
    }

    #[test]
    fn wall_of_reverence_gains_life_equal_to_power() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::wall_of_reverence());
        let big = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2 power
        let life = g.players[0].life;
        g.active_player_idx = 0;
        g.step = TurnStep::End;
        g.fire_step_triggers(TurnStep::End);
        drain_stack(&mut g);
        assert_eq!(g.players[0].life, life + 2, "gained life = target creature's power");
        let _ = big;
    }

    #[test]
    fn grim_guardian_drains_on_enchantment_etb() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::grim_guardian());
        let life = g.players[1].life;
        // A second enchantment entering triggers constellation.
        let aura = g.add_card_to_hand(0, catalog::nyx_fleece_ram()); // enchantment creature
        g.players[0].mana_pool.add(crabomination::mana::Color::White, 1);
        g.players[0].mana_pool.add_colorless(1);
        cast(&mut g, aura);
        assert_eq!(g.players[1].life, life - 1, "each opponent lost 1 from constellation");
    }

    #[test]
    fn underworld_coinsmith_activated_drain() {
        let mut g = two_player_game();
        let cs = g.add_card_to_battlefield(0, catalog::underworld_coinsmith());
        let opp = g.players[1].life;
        let me = g.players[0].life;
        g.players[0].mana_pool.add(crabomination::mana::Color::White, 1);
        g.players[0].mana_pool.add(crabomination::mana::Color::Black, 1);
        g.perform_action(GameAction::ActivateAbility {
            card_id: cs, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
        }).expect("activate Coinsmith");
        drain_stack(&mut g);
        assert_eq!(g.players[1].life, opp - 1, "opponent lost 1");
        assert_eq!(g.players[0].life, me - 1, "paid 1 life");
    }

    #[test]
    fn fecundity_draws_when_a_creature_dies() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::fecundity());
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        for _ in 0..2 { g.add_card_to_library(0, catalog::plains()); }
        // Bolt my own creature so the full SBA+dispatch path fires Fecundity.
        let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
        g.players[0].mana_pool.add(crabomination::mana::Color::Red, 1);
        let hand = g.players[0].hand.len();
        cast_at(&mut g, bolt, Target::Permanent(bear));
        assert_eq!(g.players[0].hand.len(), hand - 1 + 1, "bolt cast, then drew when the creature died");
    }

    #[test]
    fn mask_of_griselbrand_draws_on_equipped_death() {
        let mut g = two_player_game();
        let mask = g.add_card_to_battlefield(0, catalog::mask_of_griselbrand());
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
        for _ in 0..3 { g.add_card_to_library(0, catalog::plains()); }
        g.players[0].mana_pool.add_colorless(3);
        g.perform_action(GameAction::Equip { equipment: mask, target: bear }).expect("equip");
        drain_stack(&mut g);
        assert!(g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Lifelink),
            "equipped creature has lifelink");
        let hand = g.players[0].hand.len();
        g.remove_to_graveyard_with_triggers(bear);
        drain_stack(&mut g);
        assert_eq!(g.players[0].hand.len(), hand + 2, "drew cards equal to its power");
    }

    #[test]
    fn sanctuary_cat_is_a_one_two() {
        let mut g = two_player_game();
        let cat = g.add_card_to_battlefield(0, catalog::sanctuary_cat());
        let cp = g.computed_permanent(cat).unwrap();
        assert_eq!((cp.power, cp.toughness), (1, 2));
    }

    #[test]
    fn chaplains_blessing_gains_five() {
        let mut g = two_player_game();
        let id = g.add_card_to_hand(0, catalog::chaplains_blessing());
        let life = g.players[0].life;
        g.players[0].mana_pool.add(crabomination::mana::Color::White, 1);
        cast(&mut g, id);
        assert_eq!(g.players[0].life, life + 5);
    }

    #[test]
    fn vicious_hunger_pings_and_gains() {
        let mut g = two_player_game();
        let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let id = g.add_card_to_hand(0, catalog::vicious_hunger());
        let life = g.players[0].life;
        g.players[0].mana_pool.add(crabomination::mana::Color::Black, 2);
        cast_at(&mut g, id, Target::Permanent(victim));
        assert!(!g.battlefield.iter().any(|c| c.id == victim), "2 damage killed the 2/2");
        assert_eq!(g.players[0].life, life + 2, "gained 2 life");
    }

    #[test]
    fn life_goes_on_scales_with_a_death() {
        let mut g = two_player_game();
        // No creature died: gain 4.
        let id = g.add_card_to_hand(0, catalog::life_goes_on());
        let life = g.players[0].life;
        g.players[0].mana_pool.add(crabomination::mana::Color::Green, 1);
        cast(&mut g, id);
        assert_eq!(g.players[0].life, life + 4, "gained 4 with no death");
        // Now with a death this turn: gain 8.
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.remove_to_graveyard_with_triggers(bear);
        let id2 = g.add_card_to_hand(0, catalog::life_goes_on());
        let life2 = g.players[0].life;
        g.players[0].mana_pool.add(crabomination::mana::Color::Green, 1);
        cast(&mut g, id2);
        assert_eq!(g.players[0].life, life2 + 8, "gained 8 after a creature died");
    }

    #[test]
    fn feed_the_clan_scales_with_ferocious() {
        let mut g = two_player_game();
        let id = g.add_card_to_hand(0, catalog::feed_the_clan());
        let life = g.players[0].life;
        g.players[0].mana_pool.add(crabomination::mana::Color::Green, 1);
        g.players[0].mana_pool.add_colorless(1);
        cast(&mut g, id);
        assert_eq!(g.players[0].life, life + 5, "gained 5 without ferocious");
        // With a 4-power creature: gain 10.
        g.add_card_to_battlefield(0, catalog::craw_wurm());
        let id2 = g.add_card_to_hand(0, catalog::feed_the_clan());
        let life2 = g.players[0].life;
        g.players[0].mana_pool.add(crabomination::mana::Color::Green, 1);
        g.players[0].mana_pool.add_colorless(1);
        cast(&mut g, id2);
        assert_eq!(g.players[0].life, life2 + 10, "gained 10 with a power-4 creature");
    }

    #[test]
    fn silverflame_ritual_counters_each_creature() {
        let mut g = two_player_game();
        let a = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let b = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let id = g.add_card_to_hand(0, catalog::silverflame_ritual());
        g.players[0].mana_pool.add(crabomination::mana::Color::White, 1);
        g.players[0].mana_pool.add_colorless(3);
        cast(&mut g, id);
        assert_eq!(g.battlefield_find(a).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
        assert_eq!(g.battlefield_find(b).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
    }

    #[test]
    fn renewed_faith_gains_and_cycles() {
        let mut g = two_player_game();
        let id = g.add_card_to_hand(0, catalog::renewed_faith());
        let life = g.players[0].life;
        g.players[0].mana_pool.add(crabomination::mana::Color::White, 1);
        g.players[0].mana_pool.add_colorless(2);
        cast(&mut g, id);
        assert_eq!(g.players[0].life, life + 6, "gained 6 on cast");
        // Cycling: gain 2 via the cycle trigger.
        let id2 = g.add_card_to_hand(0, catalog::renewed_faith());
        for _ in 0..2 { g.add_card_to_library(0, catalog::plains()); }
        let life2 = g.players[0].life;
        g.players[0].mana_pool.add(crabomination::mana::Color::White, 1);
        g.players[0].mana_pool.add_colorless(1);
        g.perform_action(GameAction::Cycle { card_id: id2, x_value: None }).expect("cycle");
        drain_stack(&mut g);
        assert_eq!(g.players[0].life, life2 + 2, "gained 2 from the cycle trigger");
    }
}

mod recent82 {
    use crabomination::card::Keyword;
    use crabomination::catalog;
    use crabomination::game::two_player_game;
    use crabomination::game::types::Target;
    use crabomination::game::*;

    #[test]
    fn alloy_myr_taps_for_any_color() {
        let mut g = two_player_game();
        let myr = g.add_card_to_battlefield(0, catalog::alloy_myr());
        g.clear_sickness(myr);
        g.perform_action(GameAction::ActivateAbility {
            card_id: myr, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
        }).expect("tap Alloy Myr");
        assert!(g.players[0].mana_pool.total() >= 1, "produced a mana");
    }

    #[test]
    fn couriers_capsule_sacrifices_to_draw_two() {
        let mut g = two_player_game();
        let cap = g.add_card_to_battlefield(0, catalog::couriers_capsule());
        for _ in 0..3 { g.add_card_to_library(0, catalog::plains()); }
        let hand = g.players[0].hand.len();
        g.players[0].mana_pool.add(crabomination::mana::Color::Blue, 1);
        g.players[0].mana_pool.add_colorless(1);
        g.perform_action(GameAction::ActivateAbility {
            card_id: cap, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
        }).expect("activate Courier's Capsule");
        drain_stack(&mut g);
        assert_eq!(g.players[0].hand.len(), hand + 2, "drew two");
        assert!(!g.battlefield.iter().any(|c| c.id == cap), "capsule sacrificed");
    }

    #[test]
    fn ballista_squad_pings_for_x() {
        let mut g = two_player_game();
        let bs = g.add_card_to_battlefield(0, catalog::ballista_squad());
        g.clear_sickness(bs);
        let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
        g.players[0].mana_pool.add(crabomination::mana::Color::White, 1);
        g.players[0].mana_pool.add_colorless(2);
        g.perform_action(GameAction::ActivateAbility {
            card_id: bs, ability_index: 0, target: Some(Target::Permanent(victim)),
            additional_targets: vec![], x_value: Some(2),
        }).expect("activate Ballista Squad for X=2");
        drain_stack(&mut g);
        assert!(!g.battlefield.iter().any(|c| c.id == victim), "2 damage killed the 2/2");
    }

    #[test]
    fn gelectrode_untaps_on_instant_or_sorcery() {
        let mut g = two_player_game();
        let gel = g.add_card_to_battlefield(0, catalog::gelectrode());
        g.clear_sickness(gel);
        // Ping to tap it.
        g.perform_action(GameAction::ActivateAbility {
            card_id: gel, ability_index: 0, target: Some(Target::Player(1)),
            additional_targets: vec![], x_value: None,
        }).expect("ping");
        drain_stack(&mut g);
        assert!(g.battlefield_find(gel).unwrap().tapped, "tapped after activating");
        // Cast an instant → untaps Gelectrode.
        let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
        g.players[0].mana_pool.add(crabomination::mana::Color::Red, 1);
        cast_at(&mut g, bolt, Target::Player(1));
        assert!(!g.battlefield_find(gel).unwrap().tapped, "untapped by the I/S cast");
    }

    #[test]
    fn rally_the_peasants_pumps_team_and_has_flashback() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let id = g.add_card_to_hand(0, catalog::rally_the_peasants());
        assert!(g.find_card_anywhere(id).unwrap().definition.keywords.iter()
            .any(|k| matches!(k, Keyword::Flashback(_))), "has flashback");
        g.players[0].mana_pool.add(crabomination::mana::Color::White, 1);
        g.players[0].mana_pool.add_colorless(2);
        cast(&mut g, id);
        assert_eq!(g.computed_permanent(bear).unwrap().power, 4, "+2/+0 → 4 power");
    }

    #[test]
    fn tempered_steel_pumps_only_artifact_creatures() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::tempered_steel());
        let myr = g.add_card_to_battlefield(0, catalog::alloy_myr()); // artifact creature 2/2
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // non-artifact
        assert_eq!(g.computed_permanent(myr).unwrap().power, 4, "artifact creature gets +2/+2");
        assert_eq!(g.computed_permanent(bear).unwrap().power, 2, "non-artifact unaffected");
    }

    #[test]
    fn radiant_destiny_anthems_the_chosen_type() {
        use crabomination::card::CreatureType;
        let mut g = two_player_game();
        let rd = g.add_card_to_battlefield(0, catalog::radiant_destiny());
        g.battlefield_find_mut(rd).unwrap().chosen_creature_type = Some(CreatureType::Bear);
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        assert_eq!(g.computed_permanent(bear).unwrap().power, 3, "chosen-type Bear gets +1/+1");
    }

    #[test]
    fn fires_of_yavimaya_grants_haste_and_sacs_to_pump() {
        let mut g = two_player_game();
        let fires = g.add_card_to_battlefield(0, catalog::fires_of_yavimaya());
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        assert!(g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Haste),
            "creatures you control have haste");
        g.perform_action(GameAction::ActivateAbility {
            card_id: fires, ability_index: 0, target: Some(Target::Permanent(bear)),
            additional_targets: vec![], x_value: None,
        }).expect("sac Fires for +2/+2");
        drain_stack(&mut g);
        assert!(!g.battlefield.iter().any(|c| c.id == fires), "Fires sacrificed");
        assert_eq!(g.computed_permanent(bear).unwrap().power, 4, "+2/+2 → 4 power");
    }
}

mod recent83 {
    use crabomination::card::{CardType, Keyword};
    use crabomination::catalog;
    use crabomination::game::two_player_game;
    use crabomination::game::types::{Attack, AttackTarget, Target, TurnStep};
    use crabomination::game::*;

    fn advance_to(g: &mut GameState, step: TurnStep) {
        while g.step != step {
            g.perform_action(GameAction::PassPriority).expect("pass priority");
        }
    }

    #[test]
    fn walls_have_expected_stats_and_keywords() {
        let mut g = two_player_game();
        let kraken = g.add_card_to_battlefield(0, catalog::kraken_hatchling());
        let angelic = g.add_card_to_battlefield(0, catalog::angelic_wall());
        let steel = g.add_card_to_battlefield(0, catalog::steel_wall());
        let rampart = g.add_card_to_battlefield(0, catalog::fortified_rampart());
        assert_eq!(g.computed_permanent(kraken).unwrap().toughness, 4);
        let aw = g.computed_permanent(angelic).unwrap();
        assert!(aw.keywords.contains(&Keyword::Defender) && aw.keywords.contains(&Keyword::Flying));
        assert!(g.battlefield_find(steel).unwrap().definition.card_types.contains(&CardType::Artifact));
        assert_eq!(g.computed_permanent(rampart).unwrap().toughness, 6);
    }

    #[test]
    fn dazzling_ramparts_taps_a_creature() {
        let mut g = two_player_game();
        let dr = g.add_card_to_battlefield(0, catalog::dazzling_ramparts());
        g.clear_sickness(dr);
        let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        g.players[0].mana_pool.add(crabomination::mana::Color::White, 1);
        g.players[0].mana_pool.add_colorless(1);
        g.perform_action(GameAction::ActivateAbility {
            card_id: dr, ability_index: 0, target: Some(Target::Permanent(victim)),
            additional_targets: vec![], x_value: None,
        }).expect("tap ability");
        drain_stack(&mut g);
        assert!(g.battlefield_find(victim).unwrap().tapped, "target creature tapped");
    }

    #[test]
    fn vine_trellis_taps_for_green() {
        let mut g = two_player_game();
        let vt = g.add_card_to_battlefield(0, catalog::vine_trellis());
        g.clear_sickness(vt);
        g.perform_action(GameAction::ActivateAbility {
            card_id: vt, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
        }).expect("tap for G");
        assert_eq!(g.players[0].mana_pool.amount(crabomination::mana::Color::Green), 1);
    }

    #[test]
    fn overgrown_battlement_scales_with_defenders() {
        let mut g = two_player_game();
        let ob = g.add_card_to_battlefield(0, catalog::overgrown_battlement());
        g.add_card_to_battlefield(0, catalog::steel_wall()); // another defender
        g.add_card_to_battlefield(0, catalog::grizzly_bears()); // not a defender
        g.clear_sickness(ob);
        g.perform_action(GameAction::ActivateAbility {
            card_id: ob, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
        }).expect("tap for G per defender");
        assert_eq!(g.players[0].mana_pool.amount(crabomination::mana::Color::Green), 2,
            "two defenders → GG");
    }

    #[test]
    fn gatecreeper_vine_tutors_a_basic_land_to_hand() {
        use crabomination::decision::{DecisionAnswer, ScriptedDecider};
        let mut g = two_player_game();
        g.players[0].library.clear();
        let forest = g.add_card_to_library(0, catalog::forest());
        g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Search(Some(forest))]));
        let gv = g.add_card_to_hand(0, catalog::gatecreeper_vine());
        g.players[0].mana_pool.add(crabomination::mana::Color::Green, 1);
        g.players[0].mana_pool.add_colorless(1);
        cast(&mut g, gv);
        assert!(g.players[0].hand.iter().any(|c| c.id == forest),
            "fetched a basic land to hand");
    }

    #[test]
    fn blunt_the_assault_gains_life_and_fogs() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let attacker = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        g.clear_sickness(attacker);
        let id = g.add_card_to_hand(0, catalog::blunt_the_assault());
        let life = g.players[0].life;
        g.players[0].mana_pool.add(crabomination::mana::Color::Green, 1);
        g.players[0].mana_pool.add_colorless(3);
        cast(&mut g, id);
        assert_eq!(g.players[0].life, life + 2, "gained 1 per creature (2 on board)");
        // Opponent attacks; combat damage is prevented (fog).
        g.active_player_idx = 1;
        advance_to(&mut g, TurnStep::DeclareAttackers);
        g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker, target: AttackTarget::Player(0),
        }])).expect("attack");
        drain_stack(&mut g);
        let before = g.players[0].life;
        advance_to(&mut g, TurnStep::CombatDamage);
        drain_stack(&mut g);
        assert_eq!(g.players[0].life, before, "combat damage prevented by the fog");
    }
}

mod recent84 {
    use crabomination::card::{CounterType, CreatureType};
    use crabomination::catalog;
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    use crabomination::game::two_player_game;
    use crabomination::game::types::{Attack, AttackTarget, TurnStep};
    use crabomination::game::*;
    use crabomination::mana::Color;

    /// Choose `ct` at the next creature-type decision, then fire `id`'s ETB.
    fn enter_choosing(g: &mut GameState, id: CardId, ct: CreatureType) {
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::CreatureType(ct)]));
        g.fire_self_etb_triggers(id, 0);
        drain_stack(g);
    }

    fn cast_bear_from_hand(g: &mut GameState) {
        let id = g.add_card_to_hand(0, catalog::grizzly_bears()); // {1}{G} 2/2 Bear
        g.players[0].mana_pool.add(Color::Green, 1);
        g.players[0].mana_pool.add_colorless(1);
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
        })
        .expect("cast Grizzly Bears");
        drain_stack(g);
    }

    #[test]
    fn vanquishers_banner_anthems_and_draws_on_cast() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let banner = g.add_card_to_battlefield(0, catalog::vanquishers_banner());
        enter_choosing(&mut g, banner, CreatureType::Bear);
        let cp = g.compute_battlefield();
        let b = cp.iter().find(|c| c.id == bear).unwrap();
        assert_eq!((b.power, b.toughness), (3, 3), "chosen-type Bear gets +1/+1");

        g.add_card_to_library(0, catalog::plains());
        let hand = g.players[0].hand.len();
        cast_bear_from_hand(&mut g); // casting a Bear draws one
        assert_eq!(g.players[0].hand.len(), hand + 1, "cast-of-type drew a card (net: -Bear +draw +…)");
    }

    #[test]
    fn kindred_discovery_draws_on_enter_and_attack() {
        let mut g = two_player_game();
        let kd = g.add_card_to_battlefield(0, catalog::kindred_discovery());
        enter_choosing(&mut g, kd, CreatureType::Bear);
        g.add_card_to_library(0, catalog::plains());
        let hand = g.players[0].hand.len();
        cast_bear_from_hand(&mut g); // a Bear enters → draw
        // hand: started H, +Bear (added to hand), -Bear (cast), +1 (library plains), + drawn = H+1
        assert_eq!(g.players[0].hand.len(), hand + 1, "Bear entering drew a card");

        // Now attack with a Bear → draw again.
        let attacker = g.battlefield.iter().find(|c| c.definition.name == "Grizzly Bears").unwrap().id;
        g.clear_sickness(attacker);
        g.add_card_to_library(0, catalog::plains());
        g.active_player_idx = 0;
        g.step = TurnStep::DeclareAttackers;
        g.priority.player_with_priority = 0;
        let before = g.players[0].hand.len();
        let events = g.declare_attackers(vec![Attack { attacker, target: AttackTarget::Player(1) }])
            .expect("attack");
        g.dispatch_triggers_for_events(&events);
        drain_stack(&mut g);
        assert_eq!(g.players[0].hand.len(), before + 1, "Bear attacking drew a card");
    }

    /// CR 702.73 — a Changeling has every creature type, so it satisfies a
    /// chosen-type trigger regardless of the named type (Kindred Discovery draws
    /// when a Changeling enters even though "Bear" was chosen).
    #[test]
    fn cr_702_73_changeling_satisfies_chosen_type_trigger() {
        let mut g = two_player_game();
        let kd = g.add_card_to_battlefield(0, catalog::kindred_discovery());
        enter_choosing(&mut g, kd, CreatureType::Bear);
        g.add_card_to_library(0, catalog::plains());
        let auto = g.add_card_to_hand(0, catalog::universal_automaton()); // {2} 1/1 Changeling
        g.players[0].mana_pool.add_colorless(2);
        let hand = g.players[0].hand.len();
        g.perform_action(GameAction::CastSpell {
            card_id: auto, target: None, additional_targets: vec![], mode: None, x_value: None,
        })
        .expect("cast the Changeling");
        drain_stack(&mut g);
        assert_eq!(g.players[0].hand.len(), hand - 1 + 1, "Changeling entering drew a card");
    }

    #[test]
    fn door_of_destinies_scales_with_charge_counters() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let door = g.add_card_to_battlefield(0, catalog::door_of_destinies());
        enter_choosing(&mut g, door, CreatureType::Bear);
        // No counters yet → no pump.
        let b = g.compute_battlefield().into_iter().find(|c| c.id == bear).unwrap();
        assert_eq!((b.power, b.toughness), (2, 2), "no charge counters → no anthem");

        for _ in 0..2 { cast_bear_from_hand(&mut g); } // two Bear casts → two charge counters
        assert_eq!(g.battlefield_find(door).unwrap().counters.get(&CounterType::Charge).copied(), Some(2));
        let b = g.compute_battlefield().into_iter().find(|c| c.id == bear).unwrap();
        assert_eq!((b.power, b.toughness), (4, 4), "+1/+1 per charge counter → +2/+2");
    }
}

mod recent85 {
    use crabomination::card::{CreatureType, Keyword};
    use crabomination::catalog;
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    use crabomination::game::two_player_game;
    use crabomination::game::*;

    /// Choose `ct` at the next creature-type decision, then fire `id`'s ETB.
    fn enter_choosing(g: &mut GameState, id: CardId, ct: CreatureType) {
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::CreatureType(ct)]));
        g.fire_self_etb_triggers(id, 0);
        drain_stack(g);
    }

    #[test]
    fn steely_resolve_grants_shroud_to_chosen_type() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // Bear
        let sr = g.add_card_to_battlefield(0, catalog::steely_resolve());
        enter_choosing(&mut g, sr, CreatureType::Bear);
        let cp = g.compute_battlefield();
        assert!(cp.iter().find(|c| c.id == bear).unwrap().keywords.contains(&Keyword::Shroud),
            "chosen-type Bear has shroud");
    }

    #[test]
    fn kindred_boon_grants_indestructible() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let kb = g.add_card_to_battlefield(0, catalog::kindred_boon());
        enter_choosing(&mut g, kb, CreatureType::Bear);
        assert!(g.compute_battlefield().iter().find(|c| c.id == bear).unwrap()
            .keywords.contains(&Keyword::Indestructible), "chosen-type Bear is indestructible");
    }

    #[test]
    fn cover_of_darkness_grants_fear() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let cd = g.add_card_to_battlefield(0, catalog::cover_of_darkness());
        enter_choosing(&mut g, cd, CreatureType::Bear);
        assert!(g.compute_battlefield().iter().find(|c| c.id == bear).unwrap()
            .keywords.contains(&Keyword::Fear), "chosen-type Bear has fear");
    }

    /// CR 702.36 — Fear granted by Cover of Darkness restricts blockers through the
    /// computed-keyword combat path (a green blocker can't block; an artifact can).
    #[test]
    fn cr_702_36_granted_fear_restricts_blockers() {
        let mut g = two_player_game();
        let attacker = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // green Bear
        let cd = g.add_card_to_battlefield(0, catalog::cover_of_darkness());
        enter_choosing(&mut g, cd, CreatureType::Bear);
        let atk_kws = g.computed_permanent(attacker).unwrap().keywords.clone();
        assert!(atk_kws.contains(&Keyword::Fear), "the Bear was granted Fear");
        let check = |g: &mut GameState, def: crabomination::card::CardDefinition| -> bool {
            let blk = g.add_card_to_battlefield(1, def);
            let inst = g.battlefield_find(blk).unwrap().clone();
            let cp = g.computed_permanent(blk).unwrap();
            crabomination::game::can_block_attacker_computed(&inst, &cp, &atk_kws, &[], 2)
        };
        assert!(!check(&mut g, catalog::grizzly_bears()), "green creature can't block granted Fear");
        assert!(check(&mut g, catalog::ornithopter()), "artifact creature still can");
    }

    #[test]
    fn elvish_clancaller_anthems_other_elves() {
        let mut g = two_player_game();
        let elf = g.add_card_to_battlefield(0, catalog::llanowar_elves()); // Elf
        let caller = g.add_card_to_battlefield(0, catalog::elvish_clancaller());
        let cp = g.compute_battlefield();
        let e = cp.iter().find(|c| c.id == elf).unwrap();
        assert_eq!((e.power, e.toughness), (2, 2), "other Elf gets +1/+1");
        // The Clancaller does not pump itself.
        let c = cp.iter().find(|c| c.id == caller).unwrap();
        assert_eq!((c.power, c.toughness), (1, 1), "the lord excludes itself");
    }
}

mod recent86 {
    use crabomination::card::CreatureType;
    use crabomination::catalog;
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    use crabomination::game::two_player_game;
    use crabomination::game::types::Target;
    use crabomination::game::*;

    fn enter_choosing(g: &mut GameState, id: CardId, ct: CreatureType) {
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::CreatureType(ct)]));
        g.fire_self_etb_triggers(id, 0);
        drain_stack(g);
    }

    #[test]
    fn urzas_incubator_reduces_chosen_type_creature_cost() {
        let mut g = two_player_game();
        let inc = g.add_card_to_battlefield(0, catalog::urzas_incubator());
        enter_choosing(&mut g, inc, CreatureType::Bear);
        // Grizzly Bears is {1}{G}; with Urza's Incubator naming Bear, it costs {G}.
        let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
        g.players[0].mana_pool.add(crabomination::mana::Color::Green, 1);
        g.perform_action(GameAction::CastSpell {
            card_id: bear, target: None, additional_targets: vec![], mode: None, x_value: None,
        })
        .expect("cast for the reduced cost (generic {1} waived)");
        drain_stack(&mut g);
        assert!(g.battlefield.iter().any(|c| c.id == bear), "Bear resolved for one green");
    }

    #[test]
    fn heralds_horn_reduces_by_one_only() {
        let mut g = two_player_game();
        let horn = g.add_card_to_battlefield(0, catalog::heralds_horn());
        enter_choosing(&mut g, horn, CreatureType::Elf);
        // A non-Elf creature spell is NOT reduced: Grizzly Bears still needs {1}{G}.
        let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
        g.players[0].mana_pool.add(crabomination::mana::Color::Green, 1);
        assert!(g.perform_action(GameAction::CastSpell {
            card_id: bear, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).is_err(), "off-type spell isn't reduced, so one green alone is insufficient");
    }

    /// CR 601.2f / 117.7c — a cost reduction only removes generic mana; colored
    /// pips survive. Urza's Incubator naming Elf can't waive Llanowar Elves' {G}.
    #[test]
    fn cr_601_2f_reduction_is_generic_only() {
        let mut g = two_player_game();
        let inc = g.add_card_to_battlefield(0, catalog::urzas_incubator());
        enter_choosing(&mut g, inc, CreatureType::Elf);
        let elf = g.add_card_to_hand(0, catalog::llanowar_elves()); // {G}, an Elf
        g.players[0].mana_pool.add_colorless(5); // only generic mana
        assert!(g.perform_action(GameAction::CastSpell {
            card_id: elf, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).is_err(), "the {{G}} pip can't be paid with generic even under a {{2}} reduction");
    }

    #[test]
    fn seismic_assault_discards_land_for_two_damage() {
        let mut g = two_player_game();
        let sa = g.add_card_to_battlefield(0, catalog::seismic_assault());
        g.add_card_to_hand(0, catalog::mountain()); // a land to discard
        let life = g.players[1].life;
        g.perform_action(GameAction::ActivateAbility {
            card_id: sa, ability_index: 0, target: Some(Target::Player(1)),
            additional_targets: vec![], x_value: None,
        })
        .expect("discard a land, deal 2");
        drain_stack(&mut g);
        assert_eq!(g.players[1].life, life - 2, "2 damage to the opponent");
        assert!(g.players[0].hand.iter().all(|c| c.definition.name != "Mountain"),
            "the land was discarded as a cost");
    }
}

mod recent87 {
    use crabomination::card::{CreatureType, Keyword, Subtypes};
    use crabomination::catalog;
    use crabomination::game::two_player_game;
    use crabomination::mana::Color;

    /// A vanilla creature of the given types for tribal-count tests.
    fn vanilla(name: &'static str, types: Vec<CreatureType>) -> crabomination::card::CardDefinition {
        crabomination::card::CardDefinition {
            name,
            cost: crabomination::mana::cost(&[crabomination::mana::generic(1)]),
            card_types: vec![crabomination::card::CardType::Creature],
            subtypes: Subtypes { creature_types: types, ..Default::default() },
            power: 1,
            toughness: 1,
            ..Default::default()
        }
    }

    #[test]
    fn coat_of_arms_scales_with_shared_types() {
        let mut g = two_player_game();
        // Three Goblins (share "Goblin") + one lone Elf.
        let g1 = g.add_card_to_battlefield(0, vanilla("Gob A", vec![CreatureType::Goblin]));
        let g2 = g.add_card_to_battlefield(0, vanilla("Gob B", vec![CreatureType::Goblin]));
        let g3 = g.add_card_to_battlefield(1, vanilla("Gob C", vec![CreatureType::Goblin]));
        let elf = g.add_card_to_battlefield(0, vanilla("Lone Elf", vec![CreatureType::Elf]));
        g.add_card_to_battlefield(0, catalog::coat_of_arms());
        let cp = g.compute_battlefield();
        let pt = |id| { let c = cp.iter().find(|c| c.id == id).unwrap(); (c.power, c.toughness) };
        // Each Goblin shares with the other two → +2/+2 → 3/3.
        assert_eq!(pt(g1), (3, 3));
        assert_eq!(pt(g2), (3, 3));
        assert_eq!(pt(g3), (3, 3), "shared type counts across controllers");
        // The Elf shares with nobody → unchanged 1/1.
        assert_eq!(pt(elf), (1, 1));
    }

    #[test]
    fn akromas_memorial_grants_six_keywords() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.add_card_to_battlefield(0, catalog::akromas_memorial());
        let cp = g.compute_battlefield();
        let kws = &cp.iter().find(|c| c.id == bear).unwrap().keywords;
        for kw in [Keyword::Flying, Keyword::FirstStrike, Keyword::Vigilance, Keyword::Trample,
                   Keyword::Haste, Keyword::Protection(Color::Black), Keyword::Protection(Color::Red)] {
            assert!(kws.contains(&kw), "granted {kw:?}");
        }
    }
}

mod recent88 {
    use crabomination::card::Keyword;
    use crabomination::catalog;
    use crabomination::game::two_player_game;
    use crabomination::game::types::Target;
    use crabomination::game::*;
    use crabomination::mana::Color;

    fn cast(g: &mut GameState, id: CardId, target: Option<Target>, extra: Vec<Target>, x: Option<u32>) {
        g.perform_action(GameAction::CastSpell {
            card_id: id, target, additional_targets: extra, mode: None, x_value: x,
        })
        .expect("cast");
        drain_stack(g);
    }

    #[test]
    fn searing_wind_deals_five() {
        let mut g = two_player_game();
        let id = g.add_card_to_hand(0, catalog::searing_wind());
        g.players[0].mana_pool.add(Color::Red, 2);
        g.players[0].mana_pool.add_colorless(4);
        let life = g.players[1].life;
        cast(&mut g, id, Some(Target::Player(1)), vec![], None);
        assert_eq!(g.players[1].life, life - 5);
    }

    #[test]
    fn lava_burst_deals_x() {
        let mut g = two_player_game();
        let id = g.add_card_to_hand(0, catalog::lava_burst());
        g.players[0].mana_pool.add(Color::Red, 1);
        g.players[0].mana_pool.add_colorless(3);
        let life = g.players[1].life;
        cast(&mut g, id, Some(Target::Player(1)), vec![], Some(3));
        assert_eq!(g.players[1].life, life - 3, "X=3 → 3 damage");
    }

    #[test]
    fn jagged_lightning_hits_two_creatures() {
        let mut g = two_player_game();
        let a = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
        let b = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let id = g.add_card_to_hand(0, catalog::jagged_lightning());
        g.players[0].mana_pool.add(Color::Red, 2);
        g.players[0].mana_pool.add_colorless(3);
        cast(&mut g, id, Some(Target::Permanent(a)), vec![Target::Permanent(b)], None);
        assert!(g.battlefield_find(a).is_none() && g.battlefield_find(b).is_none(),
            "3 damage killed both 2/2s");
    }

    #[test]
    fn rain_of_embers_spares_flyers() {
        let mut g = two_player_game();
        let ground = g.add_card_to_battlefield(1, catalog::llanowar_elves()); // 1/1, no flying
        let flyer = g.add_card_to_battlefield(1, catalog::suntail_hawk()); // 1/1 flying
        let id = g.add_card_to_hand(0, catalog::rain_of_embers());
        g.players[0].mana_pool.add(Color::Red, 1);
        g.players[0].mana_pool.add_colorless(2);
        cast(&mut g, id, None, vec![], None);
        assert!(g.battlefield_find(ground).is_none(), "the grounded 1/1 died");
        assert!(g.battlefield_find(flyer).is_some(), "the flyer was spared");
    }

    #[test]
    fn thunderfoot_baloth_pumps_and_tramples_team() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let baloth = g.add_card_to_battlefield(0, catalog::thunderfoot_baloth());
        let cp = g.compute_battlefield();
        let b = cp.iter().find(|c| c.id == bear).unwrap();
        assert_eq!((b.power, b.toughness), (4, 4), "other creature +2/+2");
        assert!(b.keywords.contains(&Keyword::Trample), "other creature has trample");
        // The Baloth doesn't buff itself.
        let self_ = cp.iter().find(|c| c.id == baloth).unwrap();
        assert_eq!((self_.power, self_.toughness), (5, 5));
        assert!(!self_.keywords.contains(&Keyword::Trample), "source excluded");
    }
}

mod recent89 {
    use crabomination::catalog;
    use crabomination::game::two_player_game;
    use crabomination::game::types::Target;
    use crabomination::game::*;
    use crabomination::mana::Color;

    #[test]
    fn flame_burst_and_lightning_blast_burn_face() {
        for (mk, dmg, red, genc) in [
            (catalog::flame_burst as fn() -> crabomination::card::CardDefinition, 2, 1, 1),
            (catalog::lightning_blast as fn() -> crabomination::card::CardDefinition, 4, 1, 3),
        ] {
            let mut g = two_player_game();
            let id = g.add_card_to_hand(0, mk());
            g.players[0].mana_pool.add(Color::Red, red);
            g.players[0].mana_pool.add_colorless(genc);
            let life = g.players[1].life;
            g.perform_action(GameAction::CastSpell {
                card_id: id, target: Some(Target::Player(1)), additional_targets: vec![],
                mode: None, x_value: None,
            }).expect("cast");
            drain_stack(&mut g);
            assert_eq!(g.players[1].life, life - dmg);
        }
    }

    #[test]
    fn inferno_hits_all_creatures_and_players() {
        let mut g = two_player_game();
        let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let id = g.add_card_to_hand(0, catalog::inferno());
        g.players[0].mana_pool.add(Color::Red, 2);
        g.players[0].mana_pool.add_colorless(5);
        let (l0, l1) = (g.players[0].life, g.players[1].life);
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast Inferno");
        drain_stack(&mut g);
        assert!(g.battlefield_find(mine).is_none() && g.battlefield_find(theirs).is_none(),
            "6 damage cleared both 2/2s");
        assert_eq!(g.players[0].life, l0 - 6, "each player took 6");
        assert_eq!(g.players[1].life, l1 - 6);
    }

    #[test]
    fn crater_hellion_sweeps_other_creatures_on_etb() {
        let mut g = two_player_game();
        let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let hellion = g.add_card_to_battlefield(0, catalog::crater_hellion());
        g.fire_self_etb_triggers(hellion, 0);
        drain_stack(&mut g);
        assert!(g.battlefield_find(victim).is_none(), "4 damage killed the 2/2");
        assert!(g.battlefield_find(hellion).is_some(), "the Hellion spared itself");
    }
}

mod recent90 {
    use crabomination::catalog;
    use crabomination::card::{CounterType, Keyword};
    use crabomination::game::effects::EntityRef;
    use crabomination::game::two_player_game;
    use crabomination::game::types::Target;
    use crabomination::game::*;
    use crabomination::mana::Color;

    /// Cast a Lightning Bolt from P0 at P1's face (a red instant/sorcery cast).
    fn p0_bolt_face(g: &mut GameState) {
        let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
        g.players[0].mana_pool.add(Color::Red, 1);
        g.perform_action(GameAction::CastSpell {
            card_id: bolt, target: Some(Target::Player(1)), additional_targets: vec![],
            mode: None, x_value: None,
        }).expect("bolt castable");
        drain_stack(g);
    }

    #[test]
    fn adeliz_pumps_wizards_on_instant_cast() {
        let mut g = two_player_game();
        let adeliz = g.add_card_to_battlefield(0, catalog::adeliz_the_cinder_wind());
        p0_bolt_face(&mut g);
        // Adeliz is a Wizard, so it pumps itself +1/+1 → 3/3.
        let cp = g.computed_permanent(adeliz).unwrap();
        assert_eq!((cp.power, cp.toughness), (3, 3), "Adeliz pumps Wizards +1/+1 on I/S cast");
    }

    #[test]
    fn balmor_pumps_team_and_grants_trample() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::balmor_battlemage_captain());
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        p0_bolt_face(&mut g);
        let cp = g.computed_permanent(bear).unwrap();
        assert_eq!(cp.power, 3, "team gets +1/+0");
        assert!(cp.keywords.contains(&Keyword::Trample), "team gains trample");
    }

    #[test]
    fn bloodwater_entity_bottoms_gy_spell_to_library_top() {
        use crabomination::decision::{DecisionAnswer, ScriptedDecider};
        let mut g = two_player_game();
        let bolt = g.add_card_to_graveyard(0, catalog::lightning_bolt());
        let ent = g.add_card_to_battlefield(0, catalog::bloodwater_entity());
        // Opt into the optional "may put target …".
        g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Bool(true)]));
        g.fire_self_etb_triggers(ent, 0);
        drain_stack(&mut g);
        assert!(g.players[0].graveyard.iter().all(|c| c.id != bolt), "bolt left the graveyard");
        assert_eq!(g.players[0].library.last().map(|c| c.id), Some(bolt), "bolt is on top of library");
    }

    #[test]
    fn improbable_alliance_mints_faerie_on_second_draw() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::improbable_alliance());
        for _ in 0..2 { g.add_card_to_library(0, catalog::forest()); }
        g.players[0].cards_drawn_this_turn = 0;
        let mut ev = vec![];
        g.draw_one(0, &mut ev);
        g.dispatch_triggers_for_events(&ev);
        drain_stack(&mut g);
        assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Faerie").count(), 0);
        let mut ev2 = vec![];
        g.draw_one(0, &mut ev2);
        g.dispatch_triggers_for_events(&ev2);
        drain_stack(&mut g);
        assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Faerie").count(), 1,
            "second draw mints a Faerie");
    }

    #[test]
    fn runaway_steam_kin_gains_counters_capped_at_three_then_taps_for_mana() {
        let mut g = two_player_game();
        let kin = g.add_card_to_battlefield(0, catalog::runaway_steam_kin());
        // Four red spells; the counter clause stops at three.
        for _ in 0..4 { p0_bolt_face(&mut g); }
        assert_eq!(
            g.battlefield_find(kin).unwrap().counters.get(&CounterType::PlusOnePlusOne).copied().unwrap_or(0),
            3, "capped at three +1/+1 counters");
        // Remove three counters → add {R}{R}{R}.
        g.perform_action(GameAction::ActivateAbility {
            card_id: kin, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
        }).expect("activation");
        assert_eq!(g.players[0].mana_pool.amount(Color::Red), 3, "activation adds RRR");
        assert_eq!(
            g.battlefield_find(kin).unwrap().counters.get(&CounterType::PlusOnePlusOne).copied().unwrap_or(0),
            0, "three counters removed as a cost");
    }

    #[test]
    fn harmonic_prodigy_doubles_a_shamans_trigger() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::young_pyromancer()); // Human Shaman
        g.add_card_to_battlefield(0, catalog::harmonic_prodigy());
        p0_bolt_face(&mut g);
        // Young Pyromancer's Shaman trigger fires twice → two Elementals.
        assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Elemental").count(), 2,
            "Harmonic Prodigy doubles the Shaman's token trigger");
    }

    /// CR 603.x — the subtype trigger doubler fires a matching Wizard's *non-cast*
    /// trigger an additional time (the general dispatch path, not the Magecraft
    /// one). Niv-Mizzet (a Wizard) pings on each draw; doubled → 2 damage per draw.
    #[test]
    fn cr_603_x_subtype_doubler_doubles_a_wizard_draw_trigger() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::harmonic_prodigy());
        g.add_card_to_battlefield(0, catalog::niv_mizzet_the_firemind());
        g.add_card_to_library(0, catalog::forest());
        let life = g.players[1].life;
        let mut ev = vec![];
        g.draw_one(0, &mut ev);
        g.dispatch_triggers_for_events(&ev);
        drain_stack(&mut g);
        assert_eq!(g.players[1].life, life - 2, "Niv-Mizzet's draw trigger fires twice");
    }

    #[test]
    fn spellheart_chimera_power_scales_with_instants_in_graveyard() {
        let mut g = two_player_game();
        let chimera = g.add_card_to_battlefield(0, catalog::spellheart_chimera());
        assert_eq!(g.computed_permanent(chimera).unwrap().power, 0, "empty gy → 0 power");
        g.add_card_to_graveyard(0, catalog::lightning_bolt());
        g.add_card_to_graveyard(0, catalog::lightning_bolt());
        let cp = g.computed_permanent(chimera).unwrap();
        assert_eq!((cp.power, cp.toughness), (2, 3), "power = I/S in gy, toughness fixed 3");
    }

    #[test]
    fn roil_eruption_deals_three_unkicked() {
        let mut g = two_player_game();
        let id = g.add_card_to_hand(0, catalog::roil_eruption());
        g.players[0].mana_pool.add(Color::Red, 1);
        g.players[0].mana_pool.add_colorless(1);
        let life = g.players[1].life;
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: Some(Target::Player(1)), additional_targets: vec![],
            mode: None, x_value: None,
        }).expect("cast");
        drain_stack(&mut g);
        assert_eq!(g.players[1].life, life - 3, "unkicked Roil Eruption deals 3");
    }

    #[test]
    fn naru_meha_anthems_other_wizards() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::naru_meha_master_wizard());
        let other = g.add_card_to_battlefield(0, catalog::adeliz_the_cinder_wind()); // Wizard
        let cp = g.computed_permanent(other).unwrap();
        assert_eq!((cp.power, cp.toughness), (3, 3), "other Wizard gets +1/+1 from Naru Meha");
    }

    #[test]
    fn docent_transforms_at_three_wizards() {
        let mut g = two_player_game();
        // Two Wizards already down (Dualcaster Mage is a Human Wizard).
        g.add_card_to_battlefield(0, catalog::dualcaster_mage());
        g.add_card_to_battlefield(0, catalog::dualcaster_mage());
        let docent = g.add_card_to_battlefield(0, catalog::docent_of_perfection());
        p0_bolt_face(&mut g);
        // Cast made a 3rd Wizard token → Docent transforms to Final Iteration.
        assert_eq!(g.battlefield_find(docent).unwrap().definition.name, "Final Iteration",
            "three Wizards flips Docent");
    }

    #[test]
    fn beacon_bolt_scales_with_instants_in_graveyard_and_exile() {
        let mut g = two_player_game();
        let victim = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4
        g.add_card_to_graveyard(0, catalog::lightning_bolt());
        g.add_card_to_exile(0, catalog::lightning_bolt());
        let id = g.add_card_to_hand(0, catalog::beacon_bolt());
        g.players[0].mana_pool.add(Color::Blue, 1);
        g.players[0].mana_pool.add(Color::Red, 1);
        g.players[0].mana_pool.add_colorless(1);
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: Some(Target::Permanent(victim)), additional_targets: vec![],
            mode: None, x_value: None,
        }).expect("cast");
        drain_stack(&mut g);
        // 1 in gy + 1 in exile = 2 damage on a 4/4 → survives with 2 damage marked.
        let cp = g.battlefield_find(victim).expect("survives 2 damage");
        assert_eq!(cp.damage, 2, "Beacon Bolt deals gy+exile I/S count");
    }

    #[test]
    fn archaeomancer_returns_instant_from_graveyard() {
        let mut g = two_player_game();
        let bolt = g.add_card_to_graveyard(0, catalog::lightning_bolt());
        let arch = g.add_card_to_battlefield(0, catalog::archaeomancer());
        g.fire_self_etb_triggers(arch, 0);
        drain_stack(&mut g);
        assert!(g.players[0].hand.iter().any(|c| c.id == bolt), "bolt returned to hand");
    }

    #[test]
    fn magmatic_insight_requires_a_land_pitch_and_draws_two() {
        let mut g = two_player_game();
        // A land in hand to pay the additional cost; a nonland must NOT be pitched.
        let land = g.add_card_to_hand(0, catalog::mountain());
        let keep = g.add_card_to_hand(0, catalog::lightning_bolt());
        for _ in 0..3 { g.add_card_to_library(0, catalog::forest()); }
        let id = g.add_card_to_hand(0, catalog::magmatic_insight());
        g.players[0].mana_pool.add(Color::Red, 1);
        let hand_before = g.players[0].hand.len();
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("castable with a land to pitch");
        drain_stack(&mut g);
        assert!(g.players[0].graveyard.iter().any(|c| c.id == land), "the land was discarded");
        assert!(g.players[0].hand.iter().any(|c| c.id == keep), "the nonland was kept");
        // -spell -land +2 draws = net +0 from the starting hand count.
        assert_eq!(g.players[0].hand.len(), hand_before - 2 + 2, "drew two");
    }

    #[test]
    fn niv_mizzet_pings_when_you_draw() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::niv_mizzet_the_firemind());
        g.add_card_to_library(0, catalog::forest());
        let life = g.players[1].life;
        let mut ev = vec![];
        g.draw_one(0, &mut ev);
        g.dispatch_triggers_for_events(&ev);
        drain_stack(&mut g);
        assert_eq!(g.players[1].life, life - 1, "drawing pings the opponent for 1");
    }

    #[test]
    fn cloud_sprite_can_block_only_flyers() {
        let mut g = two_player_game();
        let cs = g.add_card_to_battlefield(0, catalog::cloud_sprite());
        let kw = &g.computed_permanent(cs).unwrap().keywords;
        assert!(kw.contains(&Keyword::Flying) && kw.contains(&Keyword::CanBlockOnlyFlying));
    }

    #[test]
    fn cinder_elemental_sacrifices_for_x_damage() {
        let mut g = two_player_game();
        let ce = g.add_card_to_battlefield(0, catalog::cinder_elemental());
        g.clear_sickness(ce);
        g.players[0].mana_pool.add(Color::Red, 1);
        g.players[0].mana_pool.add_colorless(3);
        let life = g.players[1].life;
        g.perform_action(GameAction::ActivateAbility {
            card_id: ce, ability_index: 0, target: Some(Target::Player(1)),
            additional_targets: vec![], x_value: Some(3),
        }).expect("activate for X=3");
        drain_stack(&mut g);
        assert_eq!(g.players[1].life, life - 3, "X=3 damage to face");
        assert!(g.battlefield_find(ce).is_none(), "sacrificed as a cost");
    }

    #[test]
    fn living_lightning_returns_instant_on_death() {
        let mut g = two_player_game();
        let bolt = g.add_card_to_graveyard(0, catalog::lightning_bolt());
        let ll = g.add_card_to_battlefield(0, catalog::living_lightning());
        g.remove_to_graveyard_with_triggers(ll);
        drain_stack(&mut g);
        assert!(g.players[0].hand.iter().any(|c| c.id == bolt), "dying returns an I/S from gy");
    }

    #[test]
    fn needle_drop_only_hits_already_damaged_and_draws() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        {
            let c = g.battlefield_find_mut(bear).unwrap();
            c.damage = 1;
            c.dealt_damage_this_turn = true; // legal target for Needle Drop
        }
        let hand_before = g.players[0].hand.len();
        let id = g.add_card_to_hand(0, catalog::needle_drop());
        g.players[0].mana_pool.add(Color::Red, 1);
        for _ in 0..1 { g.add_card_to_library(0, catalog::forest()); }
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: Some(Target::Permanent(bear)), additional_targets: vec![],
            mode: None, x_value: None,
        }).expect("castable at a damaged creature");
        drain_stack(&mut g);
        assert!(g.battlefield_find(bear).is_none(), "2/2 with 1 marked dies to +1");
        // hand_before excluded Needle Drop; casting it and drawing 1 nets +1.
        assert_eq!(g.players[0].hand.len(), hand_before + 1, "drew a card");
    }

    #[test]
    fn rise_from_the_tides_mints_a_zombie_per_instant_in_graveyard() {
        let mut g = two_player_game();
        g.add_card_to_graveyard(0, catalog::lightning_bolt());
        g.add_card_to_graveyard(0, catalog::lightning_bolt());
        g.add_card_to_graveyard(0, catalog::grizzly_bears()); // not I/S — ignored
        let id = g.add_card_to_hand(0, catalog::rise_from_the_tides());
        g.players[0].mana_pool.add(Color::Blue, 1);
        g.players[0].mana_pool.add_colorless(5);
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast");
        drain_stack(&mut g);
        let zs: Vec<_> = g.battlefield.iter().filter(|c| c.definition.name == "Zombie").collect();
        assert_eq!(zs.len(), 2, "one tapped Zombie per I/S in gy");
        assert!(zs.iter().all(|z| z.tapped), "the Zombies enter tapped");
    }

    #[test]
    fn storm_fleet_aerialist_raid_counter() {
        // Enters with a +1/+1 counter only after you've attacked this turn.
        let mut g = two_player_game();
        g.players[0].attacked_this_turn = true;
        let a = g.add_card_to_battlefield(0, catalog::storm_fleet_aerialist());
        g.fire_self_etb_triggers(a, 0);
        drain_stack(&mut g);
        let cp = g.computed_permanent(a).unwrap();
        assert_eq!((cp.power, cp.toughness), (2, 3), "Raid → enters 2/3");
    }

    /// CR 510/119 — a player dealt *noncombat* damage fires the new
    /// `PlayerDealtNoncombatDamage` trigger; combat damage does not.
    #[test]
    fn chandras_spitfire_pumps_on_opponent_noncombat_damage() {
        let mut g = two_player_game();
        let spitfire = g.add_card_to_battlefield(0, catalog::chandras_spitfire());
        // A Lightning Bolt (noncombat) to the opponent's face.
        let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
        g.players[0].mana_pool.add(Color::Red, 1);
        g.perform_action(GameAction::CastSpell {
            card_id: bolt, target: Some(Target::Player(1)), additional_targets: vec![],
            mode: None, x_value: None,
        }).expect("bolt castable");
        drain_stack(&mut g);
        assert_eq!(g.computed_permanent(spitfire).unwrap().power, 4, "+3/+0 on opponent noncombat damage");
    }

    /// Combat damage to a player must NOT fire Chandra's Spitfire (CR 510).
    #[test]
    fn chandras_spitfire_ignores_combat_damage() {
        let mut g = two_player_game();
        let spitfire = g.add_card_to_battlefield(0, catalog::chandras_spitfire());
        let attacker = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.clear_sickness(attacker);
        g.step = TurnStep::DeclareAttackers;
        g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker, target: AttackTarget::Player(1),
        }])).unwrap();
        g.step = TurnStep::CombatDamage;
        g.resolve_combat().unwrap();
        drain_stack(&mut g);
        assert_eq!(g.computed_permanent(spitfire).unwrap().power, 1, "combat damage doesn't pump");
    }

    #[test]
    fn cinder_pyromancer_pings_and_untaps_on_red_spell() {
        let mut g = two_player_game();
        let cp = g.add_card_to_battlefield(0, catalog::cinder_pyromancer());
        g.clear_sickness(cp);
        // {T}: 1 damage to a player.
        let life = g.players[1].life;
        g.perform_action(GameAction::ActivateAbility {
            card_id: cp, ability_index: 0, target: Some(Target::Player(1)),
            additional_targets: vec![], x_value: None,
        }).expect("tap for 1");
        drain_stack(&mut g);
        assert_eq!(g.players[1].life, life - 1);
        assert!(g.battlefield_find(cp).unwrap().tapped, "tapped by its own cost");
        // Casting a red spell may untap it.
        g.decider = Box::new(crabomination::decision::ScriptedDecider::new(
            vec![crabomination::decision::DecisionAnswer::Bool(true)],
        ));
        p0_bolt_face(&mut g);
        assert!(!g.battlefield_find(cp).unwrap().tapped, "untapped after a red spell");
    }

    #[test]
    fn mystic_retrieval_returns_instant_from_graveyard() {
        let mut g = two_player_game();
        let bolt = g.add_card_to_graveyard(0, catalog::lightning_bolt());
        let id = g.add_card_to_hand(0, catalog::mystic_retrieval());
        g.players[0].mana_pool.add(Color::Blue, 1);
        g.players[0].mana_pool.add_colorless(3);
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: Some(Target::Permanent(bolt)), additional_targets: vec![],
            mode: None, x_value: None,
        }).expect("cast");
        drain_stack(&mut g);
        assert!(g.players[0].hand.iter().any(|c| c.id == bolt), "bolt back in hand");
    }

    #[test]
    fn deprive_counters_and_bounces_a_land() {
        let mut g = two_player_game();
        let land = g.add_card_to_battlefield(0, catalog::island());
        // A spell on the stack to counter (P0's own bolt, left unresolved).
        let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
        g.players[0].mana_pool.add(Color::Red, 1);
        g.perform_action(GameAction::CastSpell {
            card_id: bolt, target: Some(Target::Player(1)), additional_targets: vec![],
            mode: None, x_value: None,
        }).expect("bolt on stack");
        let dep = g.add_card_to_hand(0, catalog::deprive());
        g.players[0].mana_pool.add(Color::Blue, 2);
        g.perform_action(GameAction::CastSpell {
            card_id: dep, target: Some(Target::Permanent(bolt)), additional_targets: vec![],
            mode: None, x_value: None,
        }).expect("Deprive castable with a land to bounce");
        drain_stack(&mut g);
        assert!(g.players[0].hand.iter().any(|c| c.id == land), "the land was bounced");
        assert!(g.players[0].graveyard.iter().any(|c| c.id == bolt), "the bolt was countered");
    }

    #[test]
    fn cerebral_vortex_draws_then_burns_by_draw_count() {
        let mut g = two_player_game();
        for _ in 0..3 { g.add_card_to_library(1, catalog::forest()); }
        g.players[1].cards_drawn_this_turn = 0;
        let id = g.add_card_to_hand(0, catalog::cerebral_vortex());
        g.players[0].mana_pool.add(Color::Blue, 1);
        g.players[0].mana_pool.add(Color::Red, 1);
        g.players[0].mana_pool.add_colorless(1);
        let life = g.players[1].life;
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: Some(Target::Player(1)), additional_targets: vec![],
            mode: None, x_value: None,
        }).expect("cast");
        drain_stack(&mut g);
        // Target drew 2 this turn → takes 2 damage.
        assert_eq!(g.players[1].life, life - 2, "damage = cards drawn this turn (2)");
    }

    #[test]
    fn flamewave_invoker_burns_a_player_for_five() {
        let mut g = two_player_game();
        let inv = g.add_card_to_battlefield(0, catalog::flamewave_invoker());
        g.clear_sickness(inv);
        g.players[0].mana_pool.add(Color::Red, 1);
        g.players[0].mana_pool.add_colorless(7);
        let life = g.players[1].life;
        g.perform_action(GameAction::ActivateAbility {
            card_id: inv, ability_index: 0, target: Some(Target::Player(1)),
            additional_targets: vec![], x_value: None,
        }).expect("activate");
        drain_stack(&mut g);
        assert_eq!(g.players[1].life, life - 5, "5 damage to the player");
    }

    #[test]
    fn goblin_taskmaster_pumps_a_goblin() {
        let mut g = two_player_game();
        let tm = g.add_card_to_battlefield(0, catalog::goblin_taskmaster());
        let other = g.add_card_to_battlefield(0, catalog::goblin_taskmaster());
        g.clear_sickness(tm);
        g.players[0].mana_pool.add(Color::Red, 1);
        g.players[0].mana_pool.add_colorless(1);
        g.perform_action(GameAction::ActivateAbility {
            card_id: tm, ability_index: 0, target: Some(Target::Permanent(other)),
            additional_targets: vec![], x_value: None,
        }).expect("pump a Goblin");
        drain_stack(&mut g);
        assert_eq!(g.computed_permanent(other).unwrap().power, 2, "+1/+0 to a Goblin");
    }

    #[test]
    fn fireslinger_pings_target_and_self() {
        let mut g = two_player_game();
        let fs = g.add_card_to_battlefield(0, catalog::fireslinger());
        g.clear_sickness(fs);
        let (l0, l1) = (g.players[0].life, g.players[1].life);
        g.perform_action(GameAction::ActivateAbility {
            card_id: fs, ability_index: 0, target: Some(Target::Player(1)),
            additional_targets: vec![], x_value: None,
        }).expect("tap");
        drain_stack(&mut g);
        assert_eq!(g.players[1].life, l1 - 1, "1 to target");
        assert_eq!(g.players[0].life, l0 - 1, "1 to you");
    }

    #[test]
    fn jackal_pup_reflects_damage_to_you() {
        let mut g = two_player_game();
        let pup = g.add_card_to_battlefield(0, catalog::jackal_pup());
        let life = g.players[0].life;
        let mut ev = vec![];
        g.deal_damage_to_from(EntityRef::Permanent(pup), 1, None, &mut ev);
        g.dispatch_triggers_for_events(&ev);
        drain_stack(&mut g);
        assert_eq!(g.players[0].life, life - 1, "reflects the damage to its controller");
    }

    #[test]
    fn rummaging_goblin_loots() {
        let mut g = two_player_game();
        let rg = g.add_card_to_battlefield(0, catalog::rummaging_goblin());
        g.clear_sickness(rg);
        let pitch = g.add_card_to_hand(0, catalog::mountain());
        g.add_card_to_library(0, catalog::forest());
        g.perform_action(GameAction::ActivateAbility {
            card_id: rg, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
        }).expect("loot");
        drain_stack(&mut g);
        assert!(g.players[0].graveyard.iter().any(|c| c.id == pitch), "discarded a card");
        assert!(g.players[0].hand.iter().any(|c| c.definition.name == "Forest"), "drew a card");
    }

    #[test]
    fn peel_from_reality_bounces_one_of_each() {
        let mut g = two_player_game();
        let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let id = g.add_card_to_hand(0, catalog::peel_from_reality());
        g.players[0].mana_pool.add(Color::Blue, 1);
        g.players[0].mana_pool.add_colorless(1);
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: Some(Target::Permanent(mine)),
            additional_targets: vec![Target::Permanent(theirs)], mode: None, x_value: None,
        }).expect("cast");
        drain_stack(&mut g);
        assert!(g.battlefield_find(mine).is_none() && g.battlefield_find(theirs).is_none(),
            "both creatures returned to hand");
    }

    #[test]
    fn consume_spirit_drains_for_x() {
        let mut g = two_player_game();
        let id = g.add_card_to_hand(0, catalog::consume_spirit());
        g.players[0].mana_pool.add(Color::Black, 3); // {2}{1}{B} for X=2
        g.players[0].mana_pool.add_colorless(1);
        let (l0, l1) = (g.players[0].life, g.players[1].life);
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: Some(Target::Player(1)), additional_targets: vec![],
            mode: None, x_value: Some(2),
        }).expect("cast X=2");
        drain_stack(&mut g);
        assert_eq!(g.players[1].life, l1 - 2, "X=2 damage");
        assert_eq!(g.players[0].life, l0 + 2, "gain X life");
    }

    #[test]
    fn vessel_of_nascency_digs_four_and_fills_graveyard() {
        let mut g = two_player_game();
        let v = g.add_card_to_battlefield(0, catalog::vessel_of_nascency());
        for _ in 0..4 { g.add_card_to_library(0, catalog::forest()); }
        g.players[0].mana_pool.add(Color::Green, 1);
        g.players[0].mana_pool.add_colorless(1);
        let gy_before = g.players[0].graveyard.len();
        g.perform_action(GameAction::ActivateAbility {
            card_id: v, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
        }).expect("sac to dig");
        drain_stack(&mut g);
        // Took one of four to hand; the other three (+ the sacrificed Vessel) hit gy.
        assert!(g.players[0].hand.iter().any(|c| c.definition.name == "Forest"), "kept one");
        assert!(g.players[0].graveyard.len() >= gy_before + 3, "rest milled");
    }

    #[test]
    fn skywinder_drake_and_ridgetop_raptor_have_their_keywords() {
        let mut g = two_player_game();
        let d = g.add_card_to_battlefield(0, catalog::skywinder_drake());
        let r = g.add_card_to_battlefield(0, catalog::ridgetop_raptor());
        let dk = &g.computed_permanent(d).unwrap().keywords;
        assert!(dk.contains(&Keyword::Flying) && dk.contains(&Keyword::CanBlockOnlyFlying));
        assert!(g.computed_permanent(r).unwrap().keywords.contains(&Keyword::DoubleStrike));
        let p = g.add_card_to_battlefield(0, catalog::cloud_pirates());
        let pk = &g.computed_permanent(p).unwrap().keywords;
        assert!(pk.contains(&Keyword::Flying) && pk.contains(&Keyword::CanBlockOnlyFlying));
    }

    #[test]
    fn warden_of_evos_isle_discounts_flying_creatures() {
        use crabomination::game::actions::cost_reduction_for_spell;
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::warden_of_evos_isle());
        // A flying creature (Serra Angel) is discounted {1}; a nonflyer isn't.
        let angel = crabomination::card::CardInstance::new(g.next_id(), catalog::serra_angel(), 0);
        let bears = crabomination::card::CardInstance::new(g.next_id(), catalog::grizzly_bears(), 0);
        assert_eq!(cost_reduction_for_spell(&g, 0, &angel, None), 1, "flying creature −1");
        assert_eq!(cost_reduction_for_spell(&g, 0, &bears, None), 0, "nonflyer unaffected");
    }
}
