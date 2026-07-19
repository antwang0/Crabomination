//! Tests for recentN card batches 208-222 (merged from per-batch micro-files).

mod recent208 {
    use crabomination::card::Keyword;
    use crabomination::catalog;
    use crabomination::game::types::{Attack, AttackTarget, Target};
    use crabomination::game::*;
    use crabomination::game::{drain_stack, two_player_game};
    use crabomination::mana::Color;

    /// Vanilla bodies carry their printed keyword set (Gleaming Barrier = Defender).
    #[test]
    fn vanillas_and_wall() {
        let mut g = two_player_game();
        let hb = g.add_card_to_battlefield(0, catalog::highborn_vampire());
        let sg = g.add_card_to_battlefield(0, catalog::swab_goblin());
        let gb = g.add_card_to_battlefield(0, catalog::gleaming_barrier());
        assert_eq!((g.computed_permanent(hb).unwrap().power, g.computed_permanent(hb).unwrap().toughness), (4, 3));
        assert_eq!((g.computed_permanent(sg).unwrap().power, g.computed_permanent(sg).unwrap().toughness), (2, 2));
        assert!(g.computed_permanent(gb).unwrap().keywords.contains(&Keyword::Defender));
    }

    /// Gleaming Barrier leaves a Treasure when it dies.
    #[test]
    fn gleaming_barrier_dies_into_treasure() {
        let mut g = two_player_game();
        let gb = g.add_card_to_battlefield(0, catalog::gleaming_barrier());
        g.battlefield_find_mut(gb).unwrap().counters.insert(crabomination::card::CounterType::MinusOneMinusOne, 4);
        g.check_state_based_actions();
        drain_stack(&mut g);
        assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Treasure").count(), 1);
    }

    /// Storm Fleet Spy draws only if you attacked this turn (Raid).
    #[test]
    fn storm_fleet_spy_raid_draw() {
        // No attack this turn → no draw.
        let mut g = two_player_game();
        g.add_card_to_library(0, catalog::island());
        let h0 = g.players[0].hand.len();
        g.move_card_to_battlefield_for_test(0, catalog::storm_fleet_spy());
        drain_stack(&mut g);
        assert_eq!(g.players[0].hand.len(), h0, "no attack → no Raid draw");

        // With an attack recorded, it draws.
        let mut g = two_player_game();
        let atk = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.clear_sickness(atk);
        g.step = TurnStep::DeclareAttackers;
        g.priority.player_with_priority = 0;
        g.declare_attackers(vec![Attack { attacker: atk, target: AttackTarget::Player(1) }]).expect("attack");
        g.add_card_to_library(0, catalog::island());
        let h1 = g.players[0].hand.len();
        g.move_card_to_battlefield_for_test(0, catalog::storm_fleet_spy());
        drain_stack(&mut g);
        assert_eq!(g.players[0].hand.len(), h1 + 1, "attacked → Raid draws");
    }

    /// Battle-Rattle Shaman pumps a target at the start of your combat.
    #[test]
    fn battle_rattle_shaman_begin_combat_pump() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::battle_rattle_shaman());
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.active_player_idx = 0;
        g.decider = Box::new(crabomination::decision::ScriptedDecider::new([
            crabomination::decision::DecisionAnswer::Bool(true),
            crabomination::decision::DecisionAnswer::Target(Target::Permanent(bear)),
        ]));
        g.fire_step_triggers(TurnStep::BeginCombat);
        drain_stack(&mut g);
        assert_eq!(g.computed_permanent(bear).unwrap().power, 4, "+2/+0 from the shaman");
    }

    /// Wildheart Invoker gives +5/+5 and trample for {8}.
    #[test]
    fn wildheart_invoker_overruns_one() {
        let mut g = two_player_game();
        let inv = g.add_card_to_battlefield(0, catalog::wildheart_invoker());
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.players[0].mana_pool.add_colorless(8);
        g.perform_action(GameAction::ActivateAbility {
            card_id: inv, ability_index: 0, target: Some(Target::Permanent(bear)),
            additional_targets: vec![], x_value: None,
        }).expect("activate");
        drain_stack(&mut g);
        let cp = g.computed_permanent(bear).unwrap();
        assert_eq!((cp.power, cp.toughness), (7, 7));
        assert!(cp.keywords.contains(&Keyword::Trample));
    }

    /// Devout Decree exiles a red creature and scries.
    #[test]
    fn devout_decree_exiles_red() {
        let mut g = two_player_game();
        let goblin = g.add_card_to_battlefield(1, catalog::swab_goblin()); // red
        g.add_card_to_library(0, catalog::island());
        let s = g.add_card_to_hand(0, catalog::devout_decree());
        g.players[0].mana_pool.add(Color::White, 1);
        g.players[0].mana_pool.add_colorless(1);
        g.step = TurnStep::PreCombatMain;
        g.perform_action(GameAction::CastSpell {
            card_id: s, target: Some(Target::Permanent(goblin)), additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast");
        drain_stack(&mut g);
        assert!(g.battlefield_find(goblin).is_none(), "red creature exiled");
        assert!(g.exile.iter().any(|c| c.id == goblin), "in exile, not graveyard");
    }
}

mod recent209 {
    use crabomination::card::{CounterType, Keyword};
    use crabomination::catalog;
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    use crabomination::game::types::{Attack, AttackTarget, Target};
    use crabomination::game::*;
    use crabomination::game::{drain_stack, two_player_game};
    use crabomination::mana::Color;

    fn advance_to(g: &mut GameState, step: TurnStep) {
        while g.step != step {
            g.perform_action(GameAction::PassPriority).expect("pass priority");
        }
    }

    /// Giant Cindermaw locks life gain for every player.
    #[test]
    fn giant_cindermaw_locks_lifegain() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::giant_cindermaw());
        assert_eq!(g.adjust_life(0, 5), g.players[0].life, "controller can't gain");
        assert_eq!(g.adjust_life(1, 5), g.players[1].life, "opponent can't gain either");
        assert_eq!(g.players[0].life, 20);
        assert_eq!(g.players[1].life, 20);
    }

    /// Feldon's Cane exiles itself and shuffles the graveyard back into the library.
    #[test]
    fn feldons_cane_recycles_graveyard() {
        let mut g = two_player_game();
        let cane = g.add_card_to_battlefield(0, catalog::feldons_cane());
        g.add_card_to_graveyard(0, catalog::grizzly_bears());
        g.add_card_to_graveyard(0, catalog::grizzly_bears());
        let lib_before = g.players[0].library.len();
        g.perform_action(GameAction::ActivateAbility {
            card_id: cane, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
        }).expect("activate Feldon's Cane");
        drain_stack(&mut g);
        assert!(g.players[0].graveyard.is_empty(), "graveyard emptied");
        assert_eq!(g.players[0].library.len(), lib_before + 2, "two cards returned to library");
        assert!(g.exile.iter().any(|c| c.id == cane), "the Cane is exiled");
    }

    /// Uncharted Haven enters tapped and records the chosen color.
    #[test]
    fn uncharted_haven_enters_tapped_with_chosen_color() {
        let mut g = two_player_game();
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Color(Color::Blue)]));
        let land = g.move_card_to_battlefield_for_test(0, catalog::uncharted_haven());
        drain_stack(&mut g);
        let c = g.battlefield_find(land).unwrap();
        assert!(c.tapped, "enters tapped");
        assert_eq!(c.chosen_color, Some(Color::Blue), "chose blue");
    }

    /// Ancestor Dragon gains 1 life per attacking creature.
    #[test]
    fn ancestor_dragon_gains_life_per_attacker() {
        let mut g = two_player_game();
        let dragon = g.add_card_to_battlefield(0, catalog::ancestor_dragon());
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.clear_sickness(dragon);
        g.clear_sickness(bear);
        g.step = TurnStep::DeclareAttackers;
        g.priority.player_with_priority = 0;
        g.declare_attackers(vec![
            Attack { attacker: dragon, target: AttackTarget::Player(1) },
            Attack { attacker: bear, target: AttackTarget::Player(1) },
        ]).expect("attack");
        drain_stack(&mut g);
        assert_eq!(g.players[0].life, 22, "gained 2 life for two attackers");
    }

    /// Jazal Goldmane pumps attackers by the number of attackers.
    #[test]
    fn jazal_goldmane_pumps_by_attacker_count() {
        let mut g = two_player_game();
        let jazal = g.add_card_to_battlefield(0, catalog::jazal_goldmane());
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.clear_sickness(jazal);
        g.clear_sickness(bear);
        g.step = TurnStep::DeclareAttackers;
        g.priority.player_with_priority = 0;
        g.declare_attackers(vec![
            Attack { attacker: jazal, target: AttackTarget::Player(1) },
            Attack { attacker: bear, target: AttackTarget::Player(1) },
        ]).expect("attack");
        g.players[0].mana_pool.add(Color::White, 2);
        g.players[0].mana_pool.add_colorless(3);
        g.perform_action(GameAction::ActivateAbility {
            card_id: jazal, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
        }).expect("pump");
        drain_stack(&mut g);
        // Two attackers → +2/+2. Grizzly Bears 2/2 → 4/4.
        let bv = g.computed_permanent(bear).unwrap();
        assert_eq!((bv.power, bv.toughness), (4, 4));
    }

    /// Ghitu Lavarunner grows and gains haste with two spells in the graveyard.
    #[test]
    fn ghitu_lavarunner_threshold() {
        let mut g = two_player_game();
        let ghitu = g.add_card_to_battlefield(0, catalog::ghitu_lavarunner());
        let base = g.computed_permanent(ghitu).unwrap();
        assert_eq!((base.power, base.toughness), (1, 2));
        assert!(!base.keywords.contains(&Keyword::Haste), "no haste with empty gy");
        g.add_card_to_graveyard(0, catalog::lightning_bolt());
        g.add_card_to_graveyard(0, catalog::lightning_bolt());
        let boosted = g.computed_permanent(ghitu).unwrap();
        assert_eq!((boosted.power, boosted.toughness), (2, 2), "+1/+0 with 2 spells");
        assert!(boosted.keywords.contains(&Keyword::Haste), "has haste");
    }

    /// Mystical Teachings tutors an instant to hand.
    #[test]
    fn mystical_teachings_tutors_instant() {
        let mut g = two_player_game();
        let bolt = g.add_card_to_library(0, catalog::lightning_bolt());
        g.add_card_to_library(0, catalog::grizzly_bears());
        let spell = g.add_card_to_hand(0, catalog::mystical_teachings());
        g.players[0].mana_pool.add(Color::Blue, 1);
        g.players[0].mana_pool.add_colorless(3);
        g.step = TurnStep::PreCombatMain;
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(bolt))]));
        g.perform_action(GameAction::CastSpell {
            card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast Mystical Teachings");
        drain_stack(&mut g);
        assert!(g.players[0].hand.iter().any(|c| c.id == bolt), "bolt tutored to hand");
    }

    /// Dragon Mage wheels both hands on combat damage.
    #[test]
    fn dragon_mage_wheels_on_combat_damage() {
        let mut g = two_player_game();
        let dm = g.add_card_to_battlefield(0, catalog::dragon_mage());
        g.clear_sickness(dm);
        for _ in 0..10 { g.add_card_to_library(0, catalog::island()); }
        for _ in 0..10 { g.add_card_to_library(1, catalog::island()); }
        g.add_card_to_hand(0, catalog::grizzly_bears());
        g.add_card_to_hand(1, catalog::grizzly_bears());
        advance_to(&mut g, TurnStep::DeclareAttackers);
        g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker: dm, target: AttackTarget::Player(1),
        }])).expect("attack");
        drain_stack(&mut g);
        advance_to(&mut g, TurnStep::CombatDamage);
        drain_stack(&mut g);
        assert_eq!(g.players[0].hand.len(), 7, "P0 wheeled to seven");
        assert_eq!(g.players[1].hand.len(), 7, "P1 wheeled to seven");
    }

    /// Time Stop ends the turn, exiling the spell beneath it on the stack.
    #[test]
    fn time_stop_ends_the_turn() {
        let mut g = two_player_game();
        g.step = TurnStep::PreCombatMain;
        let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
        g.players[0].mana_pool.add(Color::Red, 1);
        g.perform_action(GameAction::CastSpell {
            card_id: bolt, target: Some(Target::Player(1)), additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast bolt");
        let stop = g.add_card_to_hand(0, catalog::time_stop());
        g.players[0].mana_pool.add(Color::Blue, 2);
        g.players[0].mana_pool.add_colorless(4);
        g.perform_action(GameAction::CastSpell {
            card_id: stop, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast Time Stop");
        drain_stack(&mut g);
        assert!(g.exile.iter().any(|c| c.id == bolt), "the bolt was exiled by the ended turn");
        assert_eq!(g.players[1].life, 20, "bolt never resolved");
    }

    /// Fierce Empath tutors a big creature into hand.
    #[test]
    fn fierce_empath_tutors_big_creature() {
        let mut g = two_player_game();
        let djinn = g.add_card_to_library(0, catalog::mahamoti_djinn()); // MV 6
        g.add_card_to_library(0, catalog::grizzly_bears());
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(djinn))]));
        g.move_card_to_battlefield_for_test(0, catalog::fierce_empath());
        drain_stack(&mut g);
        assert!(g.players[0].hand.iter().any(|c| c.id == djinn), "MV6 creature tutored");
    }

    /// Obliterating Bolt exiles a creature it would kill.
    #[test]
    fn obliterating_bolt_exiles_lethal_target() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let spell = g.add_card_to_hand(0, catalog::obliterating_bolt());
        g.players[0].mana_pool.add(Color::Red, 1);
        g.players[0].mana_pool.add_colorless(1);
        g.step = TurnStep::PreCombatMain;
        g.perform_action(GameAction::CastSpell {
            card_id: spell, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast Obliterating Bolt");
        drain_stack(&mut g);
        assert!(g.battlefield_find(bear).is_none(), "bear gone");
        assert!(g.exile.iter().any(|c| c.id == bear), "exiled, not in graveyard");
        assert!(!g.players[1].graveyard.iter().any(|c| c.id == bear), "not in graveyard");
    }

    /// Elspeth's Smite burns an attacker and exiles it.
    #[test]
    fn elspeths_smite_exiles_attacker() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        g.clear_sickness(bear);
        g.active_player_idx = 1;
        g.step = TurnStep::DeclareAttackers;
        g.priority.player_with_priority = 1;
        g.declare_attackers(vec![Attack { attacker: bear, target: AttackTarget::Player(0) }]).expect("attack");
        let spell = g.add_card_to_hand(0, catalog::elspeths_smite());
        g.players[0].mana_pool.add(Color::White, 1);
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::CastSpell {
            card_id: spell, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast Elspeth's Smite");
        drain_stack(&mut g);
        assert!(g.exile.iter().any(|c| c.id == bear), "attacker exiled");
    }

    /// Taurean Mauler grows whenever an opponent casts a spell.
    #[test]
    fn taurean_mauler_grows_on_opponent_cast() {
        let mut g = two_player_game();
        let mauler = g.add_card_to_battlefield(0, catalog::taurean_mauler());
        let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
        g.players[1].mana_pool.add(Color::Red, 1);
        g.active_player_idx = 1;
        g.priority.player_with_priority = 1;
        g.perform_action(GameAction::CastSpell {
            card_id: bolt, target: Some(Target::Player(0)), additional_targets: vec![], mode: None, x_value: None,
        }).expect("opponent casts a spell");
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
        drain_stack(&mut g);
        assert_eq!(*g.battlefield_find(mauler).unwrap().counters.get(&CounterType::PlusOnePlusOne).unwrap_or(&0), 1);
    }
}

mod recent210 {
    use crabomination::card::CounterType;
    use crabomination::catalog;
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    use crabomination::game::types::Target;
    use crabomination::game::*;
    use crabomination::game::{drain_stack, two_player_game};
    use crabomination::mana::Color;

    /// A Guildgate enters tapped and taps for either of its two colors.
    #[test]
    fn guildgate_enters_tapped_dual() {
        let mut g = two_player_game();
        let gate = g.move_card_to_battlefield_for_test(0, catalog::izzet_guildgate());
        drain_stack(&mut g);
        assert!(g.battlefield_find(gate).unwrap().tapped, "Gate enters tapped");
        assert!(g.battlefield_find(gate).unwrap().definition.subtypes.land_types
            .contains(&crabomination::card::LandType::Gate));
        // Two mana abilities: {T}: Add {U} / {T}: Add {R}.
        assert_eq!(g.battlefield_find(gate).unwrap().definition.activated_abilities.len(), 2);
    }

    /// Heraldic Banner pumps creatures of the chosen color only.
    #[test]
    fn heraldic_banner_chosen_color_anthem() {
        let mut g = two_player_game();
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Color(Color::Green)]));
        g.move_card_to_battlefield_for_test(0, catalog::heraldic_banner());
        drain_stack(&mut g);
        let elf = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // green 2/2
        let red = g.add_card_to_battlefield(0, catalog::swab_goblin()); // red 2/2
        let ev = g.computed_permanent(elf).unwrap();
        assert_eq!((ev.power, ev.toughness), (3, 2), "green creature gets +1/+0");
        let rv = g.computed_permanent(red).unwrap();
        assert_eq!((rv.power, rv.toughness), (2, 2), "non-green creature unaffected");
    }

    /// Pirate's Cutlass auto-attaches to a Pirate on entry and buffs it.
    #[test]
    fn pirates_cutlass_attaches_to_pirate() {
        let mut g = two_player_game();
        let pirate = g.add_card_to_battlefield(0, catalog::swab_goblin()); // Goblin Pirate 2/2
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Target(Target::Permanent(pirate))]));
        g.move_card_to_battlefield_for_test(0, catalog::pirates_cutlass());
        drain_stack(&mut g);
        let pv = g.computed_permanent(pirate).unwrap();
        assert_eq!((pv.power, pv.toughness), (4, 3), "Pirate gets +2/+1 from the Cutlass");
    }

    /// Adventuring Gear pumps its wearer whenever a land enters.
    #[test]
    fn adventuring_gear_landfall_pump() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let gear = g.add_card_to_battlefield(0, catalog::adventuring_gear());
        g.step = TurnStep::PreCombatMain;
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        g.players[0].mana_pool.add_colorless(1);
        g.perform_action(GameAction::Equip { equipment: gear, target: bear }).expect("equip");
        drain_stack(&mut g);
        let land = g.add_card_to_hand(0, catalog::forest());
        g.perform_action(GameAction::PlayLand(land)).expect("play a land");
        drain_stack(&mut g);
        let bv = g.computed_permanent(bear).unwrap();
        assert_eq!((bv.power, bv.toughness), (4, 4), "landfall gives +2/+2");
    }

    /// Gnarlback Rhino draws when you target it with your own spell.
    #[test]
    fn gnarlback_rhino_draws_on_self_target() {
        let mut g = two_player_game();
        let rhino = g.add_card_to_battlefield(0, catalog::gnarlback_rhino());
        let pump = g.add_card_to_hand(0, catalog::giant_growth());
        g.add_card_to_library(0, catalog::forest());
        g.players[0].mana_pool.add(Color::Green, 1);
        g.step = TurnStep::PreCombatMain;
        let before = g.players[0].hand.len();
        g.perform_action(GameAction::CastSpell {
            card_id: pump, target: Some(Target::Permanent(rhino)), additional_targets: vec![], mode: None, x_value: None,
        }).expect("target the Rhino");
        drain_stack(&mut g);
        // -1 for casting Giant Growth, +1 from the draw trigger = net 0.
        assert_eq!(g.players[0].hand.len(), before);
        assert_eq!(g.players[0].graveyard.iter().filter(|c| c.definition.name == "Giant Growth").count(), 1);
    }

    /// Mold Adder grows when an opponent casts a blue or black spell, but not a red one.
    #[test]
    fn mold_adder_grows_on_blue_or_black() {
        let mut g = two_player_game();
        let adder = g.add_card_to_battlefield(0, catalog::mold_adder());
        // A red spell does nothing.
        let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
        g.players[1].mana_pool.add(Color::Red, 1);
        g.active_player_idx = 1;
        g.priority.player_with_priority = 1;
        g.perform_action(GameAction::CastSpell {
            card_id: bolt, target: Some(Target::Player(0)), additional_targets: vec![], mode: None, x_value: None,
        }).expect("red spell");
        drain_stack(&mut g);
        assert_eq!(*g.battlefield_find(adder).unwrap().counters.get(&CounterType::PlusOnePlusOne).unwrap_or(&0), 0);
        // Player 0 puts a red bolt on the stack; the opponent counters it with a
        // blue spell — that blue cast grows Mold Adder.
        g.active_player_idx = 0;
        g.step = TurnStep::PreCombatMain;
        let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
        g.players[0].mana_pool.add(Color::Red, 1);
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::CastSpell {
            card_id: bolt, target: Some(Target::Player(1)), additional_targets: vec![], mode: None, x_value: None,
        }).expect("bolt on the stack");
        let blue = g.add_card_to_hand(1, catalog::counterspell());
        g.players[1].mana_pool.add(Color::Blue, 2);
        g.priority.player_with_priority = 1;
        g.perform_action(GameAction::CastSpell {
            card_id: blue, target: Some(Target::Permanent(bolt)), additional_targets: vec![], mode: None, x_value: None,
        }).expect("opponent counters with a blue spell");
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
        drain_stack(&mut g);
        assert_eq!(*g.battlefield_find(adder).unwrap().counters.get(&CounterType::PlusOnePlusOne).unwrap_or(&0), 1);
    }
}

mod recent211 {
    use crabomination::catalog;
    use crabomination::game::types::{Attack, AttackTarget, Target};
    use crabomination::game::*;
    use crabomination::game::{drain_stack, two_player_game};
    use crabomination::mana::Color;

    fn advance_to(g: &mut GameState, step: TurnStep) {
        while g.step != step {
            g.perform_action(GameAction::PassPriority).expect("pass priority");
        }
    }

    /// Fynn turns any deathtouch creature's combat damage into two poison counters.
    #[test]
    fn fynn_grants_poison_on_deathtouch_hit() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::fynn_the_fangbearer());
        // A separate deathtouch attacker — Fynn's ability keys off any deathtoucher.
        let biter = g.add_card_to_battlefield(0, catalog::typhoid_rats());
        g.clear_sickness(biter);
        advance_to(&mut g, TurnStep::DeclareAttackers);
        g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker: biter, target: AttackTarget::Player(1),
        }])).expect("attack");
        drain_stack(&mut g);
        advance_to(&mut g, TurnStep::CombatDamage);
        drain_stack(&mut g);
        assert_eq!(g.players[1].poison_counters, 2, "deathtouch hit gave two poison");
    }

    /// River's Rebuke returns all of a player's nonland permanents to hand.
    #[test]
    fn rivers_rebuke_bounces_target_players_board() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let art = g.add_card_to_battlefield(1, catalog::feldons_cane());
        let land = g.add_card_to_battlefield(1, catalog::forest());
        let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let spell = g.add_card_to_hand(0, catalog::rivers_rebuke());
        g.players[0].mana_pool.add(Color::Blue, 2);
        g.players[0].mana_pool.add_colorless(4);
        g.step = TurnStep::PreCombatMain;
        g.perform_action(GameAction::CastSpell {
            card_id: spell, target: Some(Target::Player(1)), additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast River's Rebuke");
        drain_stack(&mut g);
        assert!(g.battlefield_find(bear).is_none(), "creature bounced");
        assert!(g.battlefield_find(art).is_none(), "artifact bounced");
        assert!(g.battlefield_find(land).is_some(), "land stays");
        assert!(g.battlefield_find(mine).is_some(), "my board untouched");
        assert_eq!(g.players[1].hand.iter().filter(|c| c.id == bear || c.id == art).count(), 2);
    }

    /// Painful Quandary drains an opponent 5 life when they can't spare a card.
    #[test]
    fn painful_quandary_drains_on_empty_hand() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::painful_quandary());
        // Opponent casts a bolt from an otherwise-empty hand → can't discard → -5.
        let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
        g.players[1].mana_pool.add(Color::Red, 1);
        g.active_player_idx = 1;
        g.priority.player_with_priority = 1;
        g.perform_action(GameAction::CastSpell {
            card_id: bolt, target: Some(Target::Player(0)), additional_targets: vec![], mode: None, x_value: None,
        }).expect("opponent casts a spell");
        drain_stack(&mut g);
        assert_eq!(g.players[1].life, 15, "no card to discard → lost 5 life");
    }

    /// Lathliss mints a 5/5 Dragon whenever another nontoken Dragon enters.
    #[test]
    fn lathliss_mints_dragon_on_dragon_etb() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::lathliss_dragon_queen());
        let before = g.battlefield.iter().filter(|c| c.definition.name == "Dragon").count();
        let dm = g.add_card_to_hand(0, catalog::dragon_mage());
        g.players[0].mana_pool.add(Color::Red, 2);
        g.players[0].mana_pool.add_colorless(5);
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::CastSpell {
            card_id: dm, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast Dragon Mage");
        drain_stack(&mut g);
        let after = g.battlefield.iter().filter(|c| c.definition.name == "Dragon" && c.controller == 0).count();
        assert_eq!(after, before + 1, "a 5/5 Dragon token was minted");
    }

    /// Bolt Bend costs {3} less while you control a creature with power 4+.
    #[test]
    fn bolt_bend_cost_reduction() {
        use crabomination::game::actions::cost_reduction_for_spell;
        let mut g = two_player_game();
        let spell = crabomination::card::CardInstance::new(g.next_id(), catalog::bolt_bend(), 0);
        assert_eq!(cost_reduction_for_spell(&g, 0, &spell, None), 0, "no big creature → no discount");
        g.add_card_to_battlefield(0, catalog::ancestor_dragon()); // 5/6
        assert_eq!(cost_reduction_for_spell(&g, 0, &spell, None), 3, "power-4+ creature → {{3}} off");
    }
}

mod recent212 {
    use crabomination::card::{CounterType, Keyword};
    use crabomination::catalog;
    use crabomination::game::types::{Attack, AttackTarget, Target};
    use crabomination::game::*;
    use crabomination::game::{drain_stack, two_player_game};
    use crabomination::mana::Color;

    fn advance_to(g: &mut GameState, step: TurnStep) {
        while g.step != step {
            g.perform_action(GameAction::PassPriority).expect("pass priority");
        }
    }

    /// Goblin Smuggler makes a small creature unblockable.
    #[test]
    fn goblin_smuggler_grants_unblockable() {
        let mut g = two_player_game();
        let smug = g.add_card_to_battlefield(0, catalog::goblin_smuggler());
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // power 2
        g.clear_sickness(smug);
        g.step = TurnStep::PreCombatMain;
        g.perform_action(GameAction::ActivateAbility {
            card_id: smug, ability_index: 0, target: Some(Target::Permanent(bear)),
            additional_targets: Vec::new(), x_value: None,
        }).expect("grant unblockable");
        drain_stack(&mut g);
        assert!(g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Unblockable));
    }

    /// Joraga Invocation pumps your team and forces blocks.
    #[test]
    fn joraga_invocation_pumps_and_lures() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let spell = g.add_card_to_hand(0, catalog::joraga_invocation());
        g.players[0].mana_pool.add(Color::Green, 2);
        g.players[0].mana_pool.add_colorless(4);
        g.step = TurnStep::PreCombatMain;
        g.perform_action(GameAction::CastSpell {
            card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast Joraga Invocation");
        drain_stack(&mut g);
        let v = g.computed_permanent(bear).unwrap();
        assert_eq!((v.power, v.toughness), (5, 5), "+3/+3");
        assert!(v.keywords.contains(&Keyword::MustBeBlocked), "must be blocked");
    }

    /// Aurelia untaps your team and grants an extra combat on her first attack.
    #[test]
    fn aurelia_untaps_and_adds_combat() {
        let mut g = two_player_game();
        let aurelia = g.add_card_to_battlefield(0, catalog::aurelia_the_warleader());
        let tapped = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.battlefield_find_mut(tapped).unwrap().tapped = true;
        g.clear_sickness(aurelia);
        advance_to(&mut g, TurnStep::DeclareAttackers);
        g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker: aurelia, target: AttackTarget::Player(1),
        }])).expect("attack");
        drain_stack(&mut g);
        assert!(!g.battlefield_find(tapped).unwrap().tapped, "team untapped");
        assert!(g.additional_combat_phases > 0, "an extra combat phase is queued");
    }

    /// Mindsparker zaps an opponent for casting a blue instant.
    #[test]
    fn mindsparker_zaps_on_blue_instant() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::mindsparker());
        // Opponent bolts our face with a blue counterspell target… use a bolt on stack.
        g.step = TurnStep::PreCombatMain;
        let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
        g.players[0].mana_pool.add(Color::Red, 1);
        g.perform_action(GameAction::CastSpell {
            card_id: bolt, target: Some(Target::Player(1)), additional_targets: vec![], mode: None, x_value: None,
        }).expect("bolt on stack");
        let blue = g.add_card_to_hand(1, catalog::counterspell());
        g.players[1].mana_pool.add(Color::Blue, 2);
        g.priority.player_with_priority = 1;
        g.perform_action(GameAction::CastSpell {
            card_id: blue, target: Some(Target::Permanent(bolt)), additional_targets: vec![], mode: None, x_value: None,
        }).expect("opponent casts a blue spell");
        drain_stack(&mut g);
        assert_eq!(g.players[1].life, 18, "Mindsparker dealt 2 to the caster");
    }

    /// Ingenious Leonin puts a counter on an attacker and grants a Cat first strike.
    #[test]
    fn ingenious_leonin_counters_and_grants_first_strike() {
        let mut g = two_player_game();
        let leonin = g.add_card_to_battlefield(0, catalog::ingenious_leonin());
        // A second Cat attacker (Leonin can't target itself in the printed text).
        let cat = g.add_card_to_battlefield(0, catalog::ingenious_leonin());
        g.clear_sickness(cat);
        g.step = TurnStep::DeclareAttackers;
        g.priority.player_with_priority = 0;
        g.declare_attackers(vec![Attack { attacker: cat, target: AttackTarget::Player(1) }]).expect("attack");
        g.players[0].mana_pool.add(Color::White, 1);
        g.players[0].mana_pool.add_colorless(3);
        g.perform_action(GameAction::ActivateAbility {
            card_id: leonin, ability_index: 0, target: Some(Target::Permanent(cat)),
            additional_targets: Vec::new(), x_value: None,
        }).expect("pump the attacking Cat");
        drain_stack(&mut g);
        let v = g.computed_permanent(cat).unwrap();
        assert_eq!(*g.battlefield_find(cat).unwrap().counters.get(&CounterType::PlusOnePlusOne).unwrap_or(&0), 1);
        assert!(v.keywords.contains(&Keyword::FirstStrike), "Cat gained first strike");
    }

    /// Crossway Troublemakers gives attacking Vampires deathtouch + lifelink.
    #[test]
    fn crossway_buffs_attacking_vampires() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::crossway_troublemakers());
        let vamp = g.add_card_to_battlefield(0, catalog::highborn_vampire()); // 4/3 Vampire Warrior
        g.clear_sickness(vamp);
        // Not attacking yet → no grant.
        let idle = g.computed_permanent(vamp).unwrap();
        assert!(!idle.keywords.contains(&Keyword::Deathtouch), "idle Vampire has no bonus");
        g.step = TurnStep::DeclareAttackers;
        g.priority.player_with_priority = 0;
        g.declare_attackers(vec![Attack { attacker: vamp, target: AttackTarget::Player(1) }]).expect("attack");
        let atk = g.computed_permanent(vamp).unwrap();
        assert!(atk.keywords.contains(&Keyword::Deathtouch), "attacking Vampire gains deathtouch");
        assert!(atk.keywords.contains(&Keyword::Lifelink), "and lifelink");
    }
}

mod recent213 {
    use crabomination::card::{CounterType, Keyword};
    use crabomination::catalog;
    use crabomination::game::actions::cost_reduction_for_spell;
    use crabomination::game::types::Target;
    use crabomination::game::*;
    use crabomination::game::{drain_stack, two_player_game};
    use crabomination::mana::Color;

    /// Heroes' Bane enters as a 4/4 and doubles via its power-scaled pump.
    #[test]
    fn heroes_bane_enters_and_pumps_by_power() {
        let mut g = two_player_game();
        let hydra = g.move_card_to_battlefield_for_test(0, catalog::heroes_bane());
        drain_stack(&mut g);
        let v = g.computed_permanent(hydra).unwrap();
        assert_eq!((v.power, v.toughness), (4, 4), "enters with four +1/+1 counters");
        g.step = TurnStep::PreCombatMain;
        g.players[0].mana_pool.add(Color::Green, 2);
        g.players[0].mana_pool.add_colorless(2);
        g.perform_action(GameAction::ActivateAbility {
            card_id: hydra, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
        }).expect("pump by power");
        drain_stack(&mut g);
        let v = g.computed_permanent(hydra).unwrap();
        assert_eq!((v.power, v.toughness), (8, 8), "added +4/+4 (X = power 4)");
    }

    /// Wildwood Scourge grows when a +1/+1 counter lands on another non-Hydra.
    #[test]
    fn wildwood_scourge_tracks_counters() {
        let mut g = two_player_game();
        let scourge = g.add_card_to_battlefield(0, catalog::wildwood_scourge());
        g.battlefield_find_mut(scourge).unwrap().counters.insert(CounterType::PlusOnePlusOne, 2);
        let _bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        // Put a +1/+1 counter on the (non-Hydra) bear through the real add path so
        // the CounterAdded event fires.
        let ctx = crabomination::game::effects::EffectContext::for_ability(scourge, 0, None);
        let evs = g.resolve_effect(&crabomination::effect::Effect::AddCounter {
            what: crabomination::effect::Selector::EachPermanent(
                crabomination::card::SelectionRequirement::HasCreatureType(crabomination::card::CreatureType::Bear),
            ),
            kind: CounterType::PlusOnePlusOne,
            amount: crabomination::effect::Value::ONE,
        }, &ctx).unwrap();
        g.dispatch_triggers_for_events(&evs);
        drain_stack(&mut g);
        assert_eq!(g.battlefield_find(scourge).unwrap().counter_count(CounterType::PlusOnePlusOne), 3,
            "Scourge gained a counter from the bear's counter");
    }

    /// Sanguine Indulgence's discount switches on after you gain life.
    #[test]
    fn sanguine_indulgence_life_discount() {
        let mut g = two_player_game();
        let spell = crabomination::card::CardInstance::new(g.next_id(), catalog::sanguine_indulgence(), 0);
        assert_eq!(cost_reduction_for_spell(&g, 0, &spell, None), 0, "no lifegain → no discount");
        g.players[0].life_gained_this_turn = 3;
        assert_eq!(cost_reduction_for_spell(&g, 0, &spell, None), 3, "gained 3 → {{3}} off");
    }

    /// Demolition Field blows up a nonbasic land and ramps a basic.
    #[test]
    fn demolition_field_destroys_and_ramps() {
        let mut g = two_player_game();
        let field = g.add_card_to_battlefield(0, catalog::demolition_field());
        let target_land = g.add_card_to_battlefield(1, catalog::demolition_field()); // nonbasic opp land
        let forest = g.add_card_to_library(0, catalog::forest());
        g.step = TurnStep::PreCombatMain;
        g.players[0].mana_pool.add_colorless(2);
        g.decider = Box::new(crabomination::decision::ScriptedDecider::new([
            crabomination::decision::DecisionAnswer::Search(Some(forest)),
        ]));
        g.perform_action(GameAction::ActivateAbility {
            card_id: field, ability_index: 1, target: Some(Target::Permanent(target_land)),
            additional_targets: Vec::new(), x_value: None,
        }).expect("activate Demolition Field");
        drain_stack(&mut g);
        assert!(g.battlefield_find(target_land).is_none(), "opponent's nonbasic land destroyed");
        assert!(g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Forest"),
            "ramped a basic Forest to the battlefield");
    }

    /// Goblin Firebomb flashes in and can be sacrificed to destroy a permanent.
    #[test]
    fn goblin_firebomb_destroys_permanent() {
        let mut g = two_player_game();
        let bomb = g.add_card_to_battlefield(0, catalog::goblin_firebomb());
        assert!(catalog::goblin_firebomb().keywords.contains(&Keyword::Flash), "has flash");
        let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        g.step = TurnStep::PreCombatMain;
        g.players[0].mana_pool.add_colorless(7);
        g.perform_action(GameAction::ActivateAbility {
            card_id: bomb, ability_index: 0, target: Some(Target::Permanent(victim)),
            additional_targets: Vec::new(), x_value: None,
        }).expect("detonate the Firebomb");
        drain_stack(&mut g);
        assert!(g.battlefield_find(victim).is_none(), "target permanent destroyed");
        assert!(g.battlefield_find(bomb).is_none(), "Firebomb sacrificed");
    }

    /// Ajani's +1 adds a counter and his ultimate mints a Cat per life.
    #[test]
    fn ajani_plus_one_and_ultimate() {
        let mut g = two_player_game();
        let ajani = g.add_card_to_battlefield(0, catalog::ajani_caller_of_the_pride());
        assert_eq!(g.battlefield_find(ajani).unwrap().counter_count(CounterType::Loyalty), 4);
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.step = TurnStep::PreCombatMain;
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::ActivateLoyaltyAbility {
            card_id: ajani, ability_index: 0, target: Some(Target::Permanent(bear)), x_value: None,
        }).expect("Ajani +1");
        drain_stack(&mut g);
        assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
        assert_eq!(g.battlefield_find(ajani).unwrap().counter_count(CounterType::Loyalty), 5, "loyalty 4→5");
    }
}

mod recent214 {
    use crabomination::card::{CounterType, Keyword};
    use crabomination::catalog;
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    use crabomination::effect::{Effect, Selector, Value};
    use crabomination::game::types::{Attack, AttackTarget, Target};
    use crabomination::game::*;
    use crabomination::game::{drain_stack, two_player_game};
    use crabomination::mana::Color;

    fn advance_to(g: &mut GameState, step: TurnStep) {
        let mut guard = 0;
        while g.step != step && guard < 40 {
            g.perform_action(GameAction::PassPriority).expect("pass priority");
            guard += 1;
        }
    }

    /// Biogenic Upgrade distributes three counters onto a lone target, then doubles
    /// them (3 → 6) via the new `Selector::AllTargets`.
    #[test]
    fn biogenic_upgrade_distributes_then_doubles() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let spell = g.add_card_to_hand(0, catalog::biogenic_upgrade());
        g.step = TurnStep::PreCombatMain;
        g.players[0].mana_pool.add(Color::Green, 2);
        g.players[0].mana_pool.add_colorless(4);
        g.perform_action(GameAction::CastSpell {
            card_id: spell, target: Some(Target::Permanent(bear)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast Biogenic Upgrade");
        drain_stack(&mut g);
        assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne), 6,
            "3 distributed then doubled to 6");
    }

    /// Herald of Faith flies and gains 2 life when it attacks.
    #[test]
    fn herald_of_faith_flies_and_gains_life() {
        let mut g = two_player_game();
        let herald = g.add_card_to_battlefield(0, catalog::herald_of_faith());
        assert!(g.computed_permanent(herald).unwrap().keywords.contains(&Keyword::Flying));
        g.clear_sickness(herald);
        g.step = TurnStep::DeclareAttackers;
        g.priority.player_with_priority = 0;
        g.declare_attackers(vec![Attack { attacker: herald, target: AttackTarget::Player(1) }]).expect("attack");
        drain_stack(&mut g);
        assert_eq!(g.players[0].life, 22, "gained 2 on attack");
    }

    /// Arcanis draws three, then bounces itself to hand.
    #[test]
    fn arcanis_draws_and_bounces() {
        let mut g = two_player_game();
        let arcanis = g.add_card_to_battlefield(0, catalog::arcanis_the_omnipotent());
        for _ in 0..5 { g.add_card_to_library(0, catalog::forest()); }
        g.clear_sickness(arcanis);
        g.step = TurnStep::PreCombatMain;
        let before = g.players[0].hand.len();
        g.perform_action(GameAction::ActivateAbility {
            card_id: arcanis, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
        }).expect("draw three");
        drain_stack(&mut g);
        assert_eq!(g.players[0].hand.len(), before + 3, "drew three cards");
        g.players[0].mana_pool.add(Color::Blue, 2);
        g.players[0].mana_pool.add_colorless(2);
        g.perform_action(GameAction::ActivateAbility {
            card_id: arcanis, ability_index: 1, target: None, additional_targets: Vec::new(), x_value: None,
        }).expect("bounce self");
        drain_stack(&mut g);
        assert!(g.battlefield_find(arcanis).is_none(), "Arcanis left the battlefield");
        assert!(g.players[0].hand.iter().any(|c| c.id == arcanis), "returned to hand");
    }

    /// Confiscate steals control of an opponent's permanent while attached.
    #[test]
    fn confiscate_steals_control() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let aura = g.add_card_to_hand(0, catalog::confiscate());
        g.step = TurnStep::PreCombatMain;
        g.players[0].mana_pool.add(Color::Blue, 2);
        g.players[0].mana_pool.add_colorless(4);
        g.perform_action(GameAction::CastSpell {
            card_id: aura, target: Some(Target::Permanent(bear)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast Confiscate");
        drain_stack(&mut g);
        assert_eq!(g.battlefield_find(bear).unwrap().controller, 0, "control stolen");
    }

    /// Unflinching Courage grants +2/+2, trample, and lifelink.
    #[test]
    fn unflinching_courage_buffs() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let aura = g.add_card_to_hand(0, catalog::unflinching_courage());
        g.step = TurnStep::PreCombatMain;
        g.players[0].mana_pool.add(Color::Green, 1);
        g.players[0].mana_pool.add(Color::White, 1);
        g.players[0].mana_pool.add_colorless(1);
        g.perform_action(GameAction::CastSpell {
            card_id: aura, target: Some(Target::Permanent(bear)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast Unflinching Courage");
        drain_stack(&mut g);
        let v = g.computed_permanent(bear).unwrap();
        assert_eq!((v.power, v.toughness), (4, 4), "+2/+2");
        assert!(v.keywords.contains(&Keyword::Trample) && v.keywords.contains(&Keyword::Lifelink));
    }

    /// Suspicious Shambler exiles itself from the graveyard to make two Zombies.
    #[test]
    fn suspicious_shambler_recurs_zombies() {
        let mut g = two_player_game();
        let shambler = g.add_card_to_graveyard(0, catalog::suspicious_shambler());
        g.step = TurnStep::PreCombatMain;
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        g.players[0].mana_pool.add(Color::Black, 2);
        g.players[0].mana_pool.add_colorless(4);
        g.perform_action(GameAction::ActivateAbility {
            card_id: shambler, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
        }).expect("activate from graveyard");
        drain_stack(&mut g);
        assert!(g.exile.iter().any(|c| c.id == shambler), "exiled as a cost");
        let zombies = g.battlefield.iter().filter(|c| c.controller == 0 && c.definition.name == "Zombie").count();
        assert_eq!(zombies, 2, "two 2/2 Zombie tokens");
    }

    /// Kalastria Highborn drains 2 when another Vampire you control dies and you
    /// pay {B}. Killed via a real bolt so the full SBA+dispatch path fires.
    #[test]
    fn kalastria_highborn_drains_on_vampire_death() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::kalastria_highborn());
        let fodder = g.add_card_to_battlefield(0, catalog::kalastria_highborn()); // another 2/2 Vampire
        let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
        g.step = TurnStep::PreCombatMain;
        g.players[0].mana_pool.add(Color::Red, 1);
        g.players[0].mana_pool.add(Color::Black, 1); // for the "may pay {B}"
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
        g.perform_action(GameAction::CastSpell {
            card_id: bolt, target: Some(Target::Permanent(fodder)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("bolt the fodder Vampire");
        drain_stack(&mut g);
        assert_eq!(g.players[1].life, 18, "each opponent lost 2");
        assert_eq!(g.players[0].life, 22, "you gained 2");
    }

    /// Kargan Dragonrider only flies while you control a Dragon.
    #[test]
    fn kargan_flies_with_a_dragon() {
        let mut g = two_player_game();
        let kargan = g.add_card_to_battlefield(0, catalog::kargan_dragonrider());
        assert!(!g.computed_permanent(kargan).unwrap().keywords.contains(&Keyword::Flying), "no Dragon → no flying");
        g.add_card_to_battlefield(0, catalog::sprite_dragon());
        assert!(g.computed_permanent(kargan).unwrap().keywords.contains(&Keyword::Flying), "Dragon → flying");
    }

    /// Kitesail Corsair gains flying only while attacking.
    #[test]
    fn kitesail_flies_while_attacking() {
        let mut g = two_player_game();
        let corsair = g.add_card_to_battlefield(0, catalog::kitesail_corsair());
        assert!(!g.computed_permanent(corsair).unwrap().keywords.contains(&Keyword::Flying), "idle → no flying");
        g.clear_sickness(corsair);
        g.step = TurnStep::DeclareAttackers;
        g.priority.player_with_priority = 0;
        g.declare_attackers(vec![Attack { attacker: corsair, target: AttackTarget::Player(1) }]).expect("attack");
        assert!(g.computed_permanent(corsair).unwrap().keywords.contains(&Keyword::Flying), "attacking → flying");
    }

    /// Sphinx of the Final Word makes its controller's instants/sorceries
    /// uncounterable.
    #[test]
    fn sphinx_makes_your_spells_uncounterable() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::sphinx_of_the_final_word());
        let mine = crabomination::card::CardInstance::new(g.next_id(), catalog::lightning_bolt(), 0);
        let theirs = crabomination::card::CardInstance::new(g.next_id(), catalog::lightning_bolt(), 1);
        assert!(g.caster_grants_uncounterable(0, &mine), "your instant is uncounterable");
        assert!(!g.caster_grants_uncounterable(1, &theirs), "opponent's is not");
    }

    /// Drogskol Reaver draws a card whenever you gain life.
    #[test]
    fn drogskol_draws_on_lifegain() {
        let mut g = two_player_game();
        let reaver = g.add_card_to_battlefield(0, catalog::drogskol_reaver());
        g.add_card_to_library(0, catalog::forest());
        let before = g.players[0].hand.len();
        let evs = g.resolve_effect(&Effect::GainLife { who: Selector::You, amount: Value::Const(1) },
            &crabomination::game::effects::EffectContext::for_ability(reaver, 0, None)).unwrap();
        g.dispatch_triggers_for_events(&evs);
        drain_stack(&mut g);
        assert_eq!(g.players[0].hand.len(), before + 1, "drew a card off lifegain");
    }

    /// Primeval Bounty mints a 3/3 Beast when you cast a creature spell.
    #[test]
    fn primeval_bounty_mints_beast_on_creature_cast() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::primeval_bounty());
        let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
        g.step = TurnStep::PreCombatMain;
        g.players[0].mana_pool.add(Color::Green, 1);
        g.players[0].mana_pool.add_colorless(1);
        g.perform_action(GameAction::CastSpell {
            card_id: bear, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast a creature");
        drain_stack(&mut g);
        assert!(g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Beast"),
            "cast trigger made a Beast");
    }

    /// Deadly Plot's second mode reanimates a Zombie tapped.
    #[test]
    fn deadly_plot_reanimates_zombie() {
        let mut g = two_player_game();
        let zombie = g.add_card_to_graveyard(0, catalog::suspicious_shambler()); // a Zombie creature
        let spell = g.add_card_to_hand(0, catalog::deadly_plot());
        g.step = TurnStep::PreCombatMain;
        g.players[0].mana_pool.add(Color::Black, 1);
        g.players[0].mana_pool.add_colorless(3);
        g.perform_action(GameAction::CastSpell {
            card_id: spell, target: Some(Target::Permanent(zombie)),
            additional_targets: vec![], mode: Some(1), x_value: None,
        }).expect("cast Deadly Plot mode 1");
        drain_stack(&mut g);
        let z = g.battlefield_find(zombie).expect("reanimated");
        assert_eq!(z.controller, 0);
        assert!(z.tapped, "returns tapped");
    }

    /// Surrak grants haste at combat when creatures you control total 8+ power.
    #[test]
    fn surrak_grants_haste_when_formidable() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::surrak_the_hunt_caller()); // 5/4
        let vamp = g.add_card_to_battlefield(0, catalog::highborn_vampire()); // 4/3 → total 9
        g.active_player_idx = 0;
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Target(Target::Permanent(vamp))]));
        advance_to(&mut g, TurnStep::BeginCombat);
        drain_stack(&mut g);
        assert!(g.computed_permanent(vamp).unwrap().keywords.contains(&Keyword::Haste), "gained haste");
    }

    /// Gateway Sneak becomes unblockable when a Gate you control enters.
    #[test]
    fn gateway_sneak_unblockable_on_gate() {
        let mut g = two_player_game();
        let sneak = g.add_card_to_battlefield(0, catalog::gateway_sneak());
        let gate = g.add_card_to_hand(0, catalog::azorius_guildgate());
        g.step = TurnStep::PreCombatMain;
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::PlayLand(gate)).expect("play a Gate");
        drain_stack(&mut g);
        assert!(g.computed_permanent(sneak).unwrap().keywords.contains(&Keyword::Unblockable),
            "Gate entering made Gateway Sneak unblockable");
    }

    /// Shipwreck Dowser returns an instant/sorcery from your graveyard on ETB.
    #[test]
    fn shipwreck_dowser_returns_instant() {
        let mut g = two_player_game();
        let bolt = g.add_card_to_graveyard(0, catalog::lightning_bolt());
        let dowser = g.add_card_to_hand(0, catalog::shipwreck_dowser());
        g.step = TurnStep::PreCombatMain;
        g.players[0].mana_pool.add(Color::Blue, 2);
        g.players[0].mana_pool.add_colorless(3);
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Target(Target::Permanent(bolt))]));
        g.perform_action(GameAction::CastSpell {
            card_id: dowser, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast Shipwreck Dowser");
        drain_stack(&mut g);
        assert!(g.players[0].hand.iter().any(|c| c.id == bolt), "returned the bolt to hand");
    }

    /// Prayer of Binding exiles an opponent's permanent and gains 2 life.
    #[test]
    fn prayer_of_binding_exiles_and_gains_life() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let prayer = g.add_card_to_hand(0, catalog::prayer_of_binding());
        g.step = TurnStep::PreCombatMain;
        g.players[0].mana_pool.add(Color::White, 1);
        g.players[0].mana_pool.add_colorless(3);
        g.perform_action(GameAction::CastSpell {
            card_id: prayer, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast Prayer of Binding");
        drain_stack(&mut g);
        assert!(g.battlefield_find(bear).is_none(), "opponent's creature exiled");
        assert_eq!(g.players[0].life, 22, "gained 2 life");
    }

    /// Wildborn Preserver grows by X when another non-Human enters and you pay {X}.
    #[test]
    fn wildborn_preserver_grows_on_nonhuman_etb() {
        let mut g = two_player_game();
        let elf = g.add_card_to_battlefield(0, catalog::wildborn_preserver());
        let bear = g.add_card_to_hand(0, catalog::grizzly_bears()); // a Bear (non-Human)
        g.step = TurnStep::PreCombatMain;
        g.players[0].mana_pool.add(Color::Green, 1);
        g.players[0].mana_pool.add_colorless(3); // 1 for the bear + 2 for the pay
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Amount(2)]));
        g.perform_action(GameAction::CastSpell {
            card_id: bear, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast the Bear");
        drain_stack(&mut g);
        assert_eq!(g.battlefield_find(elf).unwrap().counter_count(CounterType::PlusOnePlusOne), 2,
            "paid {{2}} → two +1/+1 counters");
    }

    /// Immersturm Predator's sac ability taps it (indestructible), and the
    /// becomes-tapped triggers grow it and exile a graveyard card.
    #[test]
    fn immersturm_predator_sac_taps_and_grows() {
        let mut g = two_player_game();
        let pred = g.add_card_to_battlefield(0, catalog::immersturm_predator());
        let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let gy = g.add_card_to_graveyard(1, catalog::lightning_bolt());
        g.clear_sickness(pred);
        g.perform_action(GameAction::ActivateAbility {
            card_id: pred, ability_index: 0,
            target: Some(Target::Permanent(fodder)), additional_targets: Vec::new(), x_value: None,
        }).expect("sac another creature");
        drain_stack(&mut g);
        let cp = g.computed_permanent(pred).expect("predator alive");
        assert!(cp.keywords.contains(&Keyword::Indestructible), "gained indestructible");
        assert!(g.battlefield_find(pred).unwrap().tapped, "tapped by its own ability");
        assert_eq!(g.battlefield_find(pred).unwrap().counter_count(CounterType::PlusOnePlusOne), 1,
            "becomes-tapped grew it");
        assert!(g.exile.iter().any(|c| c.id == gy), "exiled a graveyard card");
    }

    /// Gratuitous Violence doubles a controlled creature's damage but not the
    /// opponent's (source-restricted CR 614.2 doubler).
    #[test]
    fn gratuitous_violence_doubles_only_your_creatures() {
        use crabomination::game::effects::EntityRef;
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::gratuitous_violence());
        let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        assert_eq!(g.scale_damage_to(Some(mine), EntityRef::Player(1), 2), 4, "your creature doubles");
        assert_eq!(g.scale_damage_to(Some(theirs), EntityRef::Player(0), 2), 2, "opponent's is normal");
    }
}

mod recent215 {
    use crabomination::card::{CounterType, CreatureType, Keyword};
    use crabomination::catalog;
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    use crabomination::game::types::{Attack, AttackTarget, Target};
    use crabomination::game::*;
    use crabomination::game::{drain_stack, two_player_game};
    use crabomination::game::effects::{EffectContext, EntityRef};
    use crabomination::mana::Color;

    fn advance_to(g: &mut GameState, step: TurnStep) {
        let mut guard = 0;
        while g.step != step && guard < 60 {
            g.perform_action(GameAction::PassPriority).expect("pass priority");
            guard += 1;
        }
    }

    /// Mild-Mannered Librarian's once-per-game ability turns it into a 3/3 Werewolf
    /// and draws a card. A second activation is rejected.
    #[test]
    fn mild_mannered_librarian_transforms_once() {
        let mut g = two_player_game();
        let lib = g.add_card_to_battlefield(0, catalog::mild_mannered_librarian());
        g.add_card_to_library(0, catalog::forest());
        g.clear_sickness(lib);
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g.players[0].mana_pool.add(Color::Green, 1);
        g.players[0].mana_pool.add_colorless(3);
        let hand_before = g.players[0].hand.len();
        g.perform_action(GameAction::ActivateAbility {
            card_id: lib, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
        }).expect("transform");
        drain_stack(&mut g);
        let v = g.computed_permanent(lib).unwrap();
        assert_eq!((v.power, v.toughness), (3, 3), "1/1 + two counters");
        assert!(v.subtypes.creature_types.contains(&CreatureType::Werewolf), "became a Werewolf");
        assert_eq!(g.players[0].hand.len(), hand_before + 1, "drew a card");
        g.players[0].mana_pool.add(Color::Green, 1);
        g.players[0].mana_pool.add_colorless(3);
        assert!(g.perform_action(GameAction::ActivateAbility {
            card_id: lib, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
        }).is_err(), "activate only once");
    }

    /// Mazemind Tome's fourth page counter exiles it and gains 4 life.
    #[test]
    fn mazemind_tome_cashes_out_at_four_pages() {
        let mut g = two_player_game();
        let tome = g.add_card_to_battlefield(0, catalog::mazemind_tome());
        g.add_card_to_library(0, catalog::forest());
        g.battlefield_find_mut(tome).unwrap().counters.insert(CounterType::Page, 3);
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        let life_before = g.players[0].life;
        // The scry ability adds the fourth page counter → exile + gain 4 life.
        g.perform_action(GameAction::ActivateAbility {
            card_id: tome, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
        }).expect("scry ability");
        drain_stack(&mut g);
        assert!(g.exile.iter().any(|c| c.id == tome), "exiled at 4 page counters");
        assert_eq!(g.players[0].life, life_before + 4, "gained 4 life");
    }

    /// Extravagant Replication clones a target nonland permanent at upkeep.
    #[test]
    fn extravagant_replication_copies_at_upkeep() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::extravagant_replication());
        g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.active_player_idx = 0;
        g.step = TurnStep::Untap;
        g.priority.player_with_priority = 0;
        advance_to(&mut g, TurnStep::Upkeep);
        drain_stack(&mut g);
        let bears = g.battlefield.iter().filter(|c| c.controller == 0 && c.definition.name == "Grizzly Bears").count();
        assert_eq!(bears, 2, "original bear + one token copy");
    }

    /// Lathril mints one Elf Warrior per point of combat damage dealt to a player.
    #[test]
    fn lathril_mints_elves_on_combat_damage() {
        let mut g = two_player_game();
        let lathril = g.add_card_to_battlefield(0, catalog::lathril_blade_of_the_elves());
        let effect = catalog::lathril_blade_of_the_elves().triggered_abilities[0].effect.clone();
        let ctx = EffectContext { event_amount: 2, ..EffectContext::for_trigger(lathril, 0, None, 0) };
        g.resolve_effect(&effect, &ctx).unwrap();
        let elves = g.battlefield.iter().filter(|c| c.definition.name == "Elf Warrior").count();
        assert_eq!(elves, 2, "2 combat damage → 2 Elf Warriors");
    }

    /// Lathril's tap-ten-Elves ability drains each opponent for 10.
    #[test]
    fn lathril_drains_ten_by_tapping_elves() {
        let mut g = two_player_game();
        let lathril = g.add_card_to_battlefield(0, catalog::lathril_blade_of_the_elves());
        for _ in 0..10 { g.add_card_to_battlefield(0, catalog::llanowar_elves()); }
        g.clear_sickness(lathril);
        for c in g.battlefield.iter_mut() { c.summoning_sick = false; }
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::ActivateAbility {
            card_id: lathril, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
        }).expect("tap ten elves");
        drain_stack(&mut g);
        assert_eq!(g.players[1].life, 10, "opponent lost 10");
        assert_eq!(g.players[0].life, 30, "you gained 10");
    }

    /// Ayli gains life equal to a sacrificed creature's toughness.
    #[test]
    fn ayli_gains_life_on_sacrifice() {
        let mut g = two_player_game();
        let ayli = g.add_card_to_battlefield(0, catalog::ayli_eternal_pilgrim());
        g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2 fodder
        g.clear_sickness(ayli);
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g.players[0].mana_pool.add_colorless(1);
        let life = g.players[0].life;
        g.perform_action(GameAction::ActivateAbility {
            card_id: ayli, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
        }).expect("sacrifice for life");
        drain_stack(&mut g);
        assert_eq!(g.players[0].life, life + 2, "gained 2 (bear toughness)");
    }

    /// Kykar makes a 1/1 white flying Spirit when a noncreature spell is cast
    /// (choosing the token mode).
    #[test]
    fn kykar_makes_spirit_on_noncreature_cast() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::kykar_zephyr_awakener());
        let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g.players[0].mana_pool.add(Color::Red, 1);
        // The mode is picked synchronously at trigger push (mode 0 has a target);
        // choose mode 1 (create Spirit).
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Mode(1)]));
        g.perform_action(GameAction::CastSpell {
            card_id: bolt, target: Some(Target::Player(1)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast bolt");
        drain_stack(&mut g);
        let spirits = g.battlefield.iter().filter(|c| c.definition.name == "Spirit").count();
        assert_eq!(spirits, 1, "one Spirit token");
        let sp = g.battlefield.iter().find(|c| c.definition.name == "Spirit").unwrap();
        assert!(sp.definition.keywords.contains(&Keyword::Flying), "flying Spirit");
    }

    /// Alesha grows on attack and, thanks to Raid, reanimates a small creature at
    /// her end step.
    #[test]
    fn alesha_reanimates_at_end_step() {
        let mut g = two_player_game();
        let alesha = g.add_card_to_battlefield(0, catalog::alesha_who_laughs_at_fate());
        let dead = g.add_card_to_graveyard(0, catalog::grizzly_bears()); // MV 2 ≤ power 3 after attack
        g.clear_sickness(alesha);
        g.active_player_idx = 0;
        g.step = TurnStep::DeclareAttackers;
        g.priority.player_with_priority = 0;
        g.declare_attackers(vec![Attack { attacker: alesha, target: AttackTarget::Player(1) }]).expect("attack");
        drain_stack(&mut g);
        assert_eq!(g.computed_permanent(alesha).unwrap().power, 3, "grew to 3 power on attack");
        advance_to(&mut g, TurnStep::End);
        drain_stack(&mut g);
        assert!(g.battlefield_find(dead).is_some(), "reanimated the bear at end step");
    }

    /// Garna draws when an attacking creature you control dies, and pings each
    /// opponent when a non-attacker dies.
    #[test]
    fn garna_draws_or_pings_on_death() {
        let mut g = two_player_game();
        let garna = g.add_card_to_battlefield(0, catalog::garna_bloodfist_of_keld());
        g.add_card_to_library(0, catalog::forest());
        let effect = catalog::garna_bloodfist_of_keld().triggered_abilities[0].effect.clone();
        // Non-attacker death → ping each opponent. (Dying creature is left on the
        // battlefield here so its `attacked_this_turn` is readable as trigger source.)
        let foe_life = g.players[1].life;
        let non_attacker = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let ctx = EffectContext {
            trigger_source: Some(EntityRef::Permanent(non_attacker)),
            ..EffectContext::for_trigger(garna, 0, None, 0)
        };
        g.resolve_effect(&effect, &ctx).unwrap();
        assert_eq!(g.players[1].life, foe_life - 1, "non-attacker death pings opponent");
        // Attacker death → draw a card.
        let attacker = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.battlefield_find_mut(attacker).unwrap().attacked_this_turn = true;
        let hand = g.players[0].hand.len();
        let ctx2 = EffectContext {
            trigger_source: Some(EntityRef::Permanent(attacker)),
            ..EffectContext::for_trigger(garna, 0, None, 0)
        };
        g.resolve_effect(&effect, &ctx2).unwrap();
        assert_eq!(g.players[0].hand.len(), hand + 1, "attacker death draws");
    }
}

mod recent216 {
    use crabomination::card::CounterType;
    use crabomination::catalog;
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    use crabomination::game::types::Target;
    use crabomination::game::*;
    use crabomination::game::{drain_stack, two_player_game};
    use crabomination::mana::Color;

    /// Teapot Slinger deals 2 to each opponent when you expend 4.
    #[test]
    fn teapot_slinger_expend_pings() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::teapot_slinger());
        let moose = g.add_card_to_hand(0, catalog::galewind_moose()); // {4}{G}{G} crosses expend 4
        g.players[0].mana_pool.add(Color::Green, 2);
        g.players[0].mana_pool.add_colorless(4);
        g.perform_action(GameAction::CastSpell {
            card_id: moose, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast 6-mana spell");
        drain_stack(&mut g);
        assert_eq!(g.players[1].life, 18, "expend 4 → 2 damage to the opponent");
    }

    /// Byway Barterer discards your hand and draws two on expend 4.
    #[test]
    fn byway_barterer_expend_wheels_hand() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::byway_barterer());
        let moose = g.add_card_to_hand(0, catalog::galewind_moose());
        g.add_card_to_hand(0, catalog::forest());
        g.add_card_to_hand(0, catalog::forest()); // hand cards to discard
        for _ in 0..4 { g.add_card_to_library(0, catalog::forest()); }
        g.players[0].mana_pool.add(Color::Green, 2);
        g.players[0].mana_pool.add_colorless(4);
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
        g.perform_action(GameAction::CastSpell {
            card_id: moose, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast 6-mana spell");
        drain_stack(&mut g);
        // The 2 forests in hand were discarded (moose left on cast), then drew two.
        assert_eq!(g.players[0].hand.len(), 2, "discarded whole hand, drew two");
        assert!(g.players[0].graveyard.iter().filter(|c| c.definition.name == "Forest").count() >= 2,
            "discarded forests hit the graveyard");
    }

    /// Wick's Patrol mills three, then hands a -X/-X to an opponent's creature where
    /// X is the greatest mana value among cards in your graveyard.
    #[test]
    fn wicks_patrol_debuffs_by_greatest_gy_mv() {
        let mut g = two_player_game();
        // Seed the graveyard: a 6-mana card ({4}{G}{G}) sets X = 6.
        g.add_card_to_graveyard(0, catalog::galewind_moose()); // MV 6
        let target = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
        let patrol = g.add_card_to_hand(0, catalog::wicks_patrol());
        for _ in 0..3 { g.add_card_to_library(0, catalog::forest()); }
        g.step = TurnStep::PreCombatMain;
        g.players[0].mana_pool.add(Color::Black, 2);
        g.players[0].mana_pool.add_colorless(4);
        g.perform_action(GameAction::CastSpell {
            card_id: patrol, target: Some(Target::Permanent(target)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast Wick's Patrol");
        drain_stack(&mut g);
        // Moose MV 6 is the greatest → -6/-6 kills the 2/2.
        assert!(g.battlefield_find(target).is_none(), "2/2 dies to -6/-6");
    }

    /// Maha sets opponents' creatures to base toughness 1 (power untouched, and
    /// +1/+1 counters stack on top per CR 613); your own creatures are unaffected.
    #[test]
    fn maha_sets_opponent_base_toughness_to_one() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::maha_its_feathers_night());
        let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
        let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2, yours
        let cp = g.computed_permanent(foe).unwrap();
        assert_eq!((cp.power, cp.toughness), (2, 1), "opponent's 2/2 → 2/1");
        let cm = g.computed_permanent(mine).unwrap();
        assert_eq!((cm.power, cm.toughness), (2, 2), "your own creature is unaffected");
        // A +1/+1 counter stacks on the reduced base (1 → 2 toughness, 2 → 3 power).
        g.battlefield_find_mut(foe).unwrap().add_counters(CounterType::PlusOnePlusOne, 1);
        let cp2 = g.computed_permanent(foe).unwrap();
        assert_eq!((cp2.power, cp2.toughness), (3, 2), "counter stacks on base toughness 1");
    }
}

mod recent217 {
    use crabomination::card::CounterType;
    use crabomination::catalog;
    use crabomination::game::*;
    use crabomination::game::{drain_stack, two_player_game};
    use crabomination::mana::Color;

    /// Serra Redeemer puts two +1/+1 counters on a small creature that enters.
    #[test]
    fn serra_redeemer_boosts_small_entrants() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::serra_redeemer());
        let small = g.add_card_to_hand(0, catalog::grizzly_bears()); // power 2 ≤ 2
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g.players[0].mana_pool.add(Color::Green, 1);
        g.players[0].mana_pool.add_colorless(1);
        g.perform_action(GameAction::CastSpell {
            card_id: small, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast bear");
        drain_stack(&mut g);
        assert_eq!(g.battlefield_find(small).unwrap().counter_count(CounterType::PlusOnePlusOne), 2,
            "small entrant gets two +1/+1 counters");
    }

    /// Wandertale Mentor grows on expend 4 and taps for red or green.
    #[test]
    fn wandertale_mentor_expend_and_mana() {
        let mut g = two_player_game();
        let mentor = g.add_card_to_battlefield(0, catalog::wandertale_mentor());
        let moose = g.add_card_to_hand(0, catalog::galewind_moose()); // {4}{G}{G} crosses expend 4
        g.players[0].mana_pool.add(Color::Green, 2);
        g.players[0].mana_pool.add_colorless(4);
        g.perform_action(GameAction::CastSpell {
            card_id: moose, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast 6-mana spell");
        drain_stack(&mut g);
        assert_eq!(g.battlefield_find(mentor).unwrap().counter_count(CounterType::PlusOnePlusOne), 1,
            "expend 4 → +1/+1 counter");
        // Mana ability (index 0 = red) produces {R}.
        g.clear_sickness(mentor);
        g.perform_action(GameAction::ActivateAbility {
            card_id: mentor, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
        }).expect("tap for red");
        assert_eq!(g.players[0].mana_pool.amount(Color::Red), 1, "added one red mana");
    }

    /// Starseer Mentor punishes an opponent who can't dodge (no permanent to sac,
    /// empty hand) for 3 life. Drives the trigger effect directly.
    #[test]
    fn starseer_mentor_drains_when_no_dodge() {
        use crabomination::game::effects::EffectContext;
        let mut g = two_player_game();
        let mentor = g.add_card_to_battlefield(0, catalog::starseer_mentor());
        g.players[1].hand.clear(); // no card to discard, no permanent to sacrifice
        let foe_life = g.players[1].life;
        let effect = catalog::starseer_mentor().triggered_abilities[0].effect.clone();
        let ctx = EffectContext::for_trigger(mentor, 0, None, 0);
        g.resolve_effect(&effect, &ctx).unwrap();
        assert_eq!(g.players[1].life, foe_life - 3, "no dodge available → opponent loses 3");
    }
}

mod recent218 {
    use crabomination::card::CounterType;
    use crabomination::catalog;
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    use crabomination::game::effects::EffectContext;
    use crabomination::game::types::{Attack, AttackTarget, Target};
    use crabomination::game::*;
    use crabomination::game::{drain_stack, two_player_game};
    use crabomination::mana::Color;

    /// Baylen taps three tokens to draw a card.
    #[test]
    fn baylen_taps_tokens_to_draw() {
        let mut g = two_player_game();
        let baylen = g.add_card_to_battlefield(0, catalog::baylen_the_haymaker());
        let tokens: Vec<_> = (0..3).map(|_| g.add_card_to_battlefield(0, catalog::grizzly_bears())).collect();
        for &t in &tokens { g.battlefield_find_mut(t).unwrap().is_token = true; }
        g.add_card_to_library(0, catalog::forest());
        g.clear_sickness(baylen);
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        let hand = g.players[0].hand.len();
        g.perform_action(GameAction::ActivateAbility {
            card_id: baylen, ability_index: 1, target: None, additional_targets: Vec::new(), x_value: None,
        }).expect("tap three tokens: draw");
        drain_stack(&mut g);
        assert_eq!(g.players[0].hand.len(), hand + 1, "drew a card");
        assert_eq!(tokens.iter().filter(|&&t| g.battlefield_find(t).unwrap().tapped).count(), 3, "three tokens tapped");
    }

    /// Haazda Vigilante puts a +1/+1 counter on a small creature on ETB.
    #[test]
    fn haazda_vigilante_boosts_on_etb() {
        let mut g = two_player_game();
        let small = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // power 2
        let haazda = catalog::haazda_vigilante();
        let effect = haazda.triggered_abilities[0].effect.clone();
        let hz = g.add_card_to_battlefield(0, catalog::haazda_vigilante());
        let ctx = EffectContext { targets: vec![Target::Permanent(small)], ..EffectContext::for_trigger(hz, 0, None, 0) };
        g.resolve_effect(&effect, &ctx).unwrap();
        assert_eq!(g.battlefield_find(small).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
    }

    /// Neighborhood Guardian pumps a creature when a small creature enters.
    #[test]
    fn neighborhood_guardian_pumps_on_small_entrant() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::neighborhood_guardian());
        let buddy = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let small = g.add_card_to_hand(0, catalog::grizzly_bears()); // power 2 ≤ 2 entrant
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g.players[0].mana_pool.add(Color::Green, 1);
        g.players[0].mana_pool.add_colorless(1);
        // Target the existing buddy with the pump.
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Target(Target::Permanent(buddy))]));
        g.perform_action(GameAction::CastSpell {
            card_id: small, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast small creature");
        drain_stack(&mut g);
        let v = g.computed_permanent(buddy).unwrap();
        assert_eq!((v.power, v.toughness), (3, 3), "buddy pumped +1/+1");
    }

    /// Griffnaut Tracker exiles up to two cards from a single graveyard on ETB.
    #[test]
    fn griffnaut_tracker_exiles_graveyard() {
        let mut g = two_player_game();
        let ids: Vec<_> = (0..3).map(|_| g.add_card_to_graveyard(1, catalog::grizzly_bears())).collect();
        let tracker = g.add_card_to_battlefield(0, catalog::griffnaut_tracker());
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Cards(vec![ids[0], ids[1]])]));
        let effect = catalog::griffnaut_tracker().triggered_abilities[0].effect.clone();
        let ctx = EffectContext::for_trigger(tracker, 0, None, 0);
        g.resolve_effect(&effect, &ctx).unwrap();
        assert_eq!(g.players[1].graveyard.len(), 1, "two of three graveyard cards exiled");
    }

    /// Rubblebelt Braggart suspects itself when it attacks.
    #[test]
    fn rubblebelt_braggart_suspects_on_attack() {
        let mut g = two_player_game();
        let brag = g.add_card_to_battlefield(0, catalog::rubblebelt_braggart());
        g.clear_sickness(brag);
        g.active_player_idx = 0;
        g.step = TurnStep::DeclareAttackers;
        g.priority.player_with_priority = 0;
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
        g.declare_attackers(vec![Attack { attacker: brag, target: AttackTarget::Player(1) }]).expect("attack");
        drain_stack(&mut g);
        assert!(g.battlefield_find(brag).unwrap().suspected, "suspected itself");
    }

    /// Gearbane Orangutan destroys an artifact via mode 0.
    #[test]
    fn gearbane_orangutan_destroys_artifact() {
        let mut g = two_player_game();
        let clue = g.add_card_to_battlefield(1, catalog::mazemind_tome()); // an artifact target
        let ape = g.add_card_to_battlefield(0, catalog::gearbane_orangutan());
        let effect = catalog::gearbane_orangutan().triggered_abilities[0].effect.clone();
        let ctx = EffectContext {
            mode: 0,
            targets: vec![Target::Permanent(clue)],
            ..EffectContext::for_trigger(ape, 0, None, 0)
        };
        g.resolve_effect(&effect, &ctx).unwrap();
        assert!(g.battlefield_find(clue).is_none(), "artifact destroyed");
    }
}

mod recent219 {
    use crabomination::card::CounterType;
    use crabomination::catalog;
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    use crabomination::game::effects::EffectContext;
    use crabomination::game::types::Target;
    use crabomination::game::*;
    use crabomination::game::{drain_stack, two_player_game};
    use crabomination::mana::Color;

    /// Flamewake Phoenix returns from the graveyard at combat when you're Ferocious
    /// and pay {R}.
    #[test]
    fn flamewake_phoenix_returns_when_ferocious() {
        let mut g = two_player_game();
        let phoenix = g.add_card_to_graveyard(0, catalog::flamewake_phoenix());
        // A power-4+ creature enables Ferocious.
        g.add_card_to_battlefield(0, catalog::griselbrand()); // 7/7
        g.active_player_idx = 0;
        g.players[0].mana_pool.add(Color::Red, 1);
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
        g.fire_step_triggers(TurnStep::BeginCombat);
        drain_stack(&mut g);
        assert!(g.battlefield_find(phoenix).is_some(), "phoenix returned to the battlefield");
    }

    /// Cryptic Caves draws a card when you control five or more lands.
    #[test]
    fn cryptic_caves_draws_with_five_lands() {
        let mut g = two_player_game();
        let caves = g.add_card_to_battlefield(0, catalog::cryptic_caves());
        for _ in 0..4 { g.add_card_to_battlefield(0, catalog::forest()); }
        g.add_card_to_library(0, catalog::forest());
        g.clear_sickness(caves);
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g.players[0].mana_pool.add_colorless(1);
        let hand = g.players[0].hand.len();
        g.perform_action(GameAction::ActivateAbility {
            card_id: caves, ability_index: 1, target: None, additional_targets: Vec::new(), x_value: None,
        }).expect("sac to draw with five lands");
        drain_stack(&mut g);
        assert_eq!(g.players[0].hand.len(), hand + 1, "drew a card");
        assert!(g.battlefield_find(caves).is_none(), "land sacrificed");
    }

    /// New Horizons puts a +1/+1 counter on a creature you control when it enters.
    #[test]
    fn new_horizons_boosts_on_etb() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let aura = catalog::new_horizons();
        let effect = aura.triggered_abilities[0].effect.clone();
        let nh = g.add_card_to_battlefield(0, catalog::new_horizons());
        let ctx = EffectContext { targets: vec![Target::Permanent(bear)], ..EffectContext::for_trigger(nh, 0, None, 0) };
        g.resolve_effect(&effect, &ctx).unwrap();
        assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
    }

    /// Drake Hatcher removes three incubation counters to mint a 2/2 flying Drake.
    #[test]
    fn drake_hatcher_hatches_a_drake() {
        let mut g = two_player_game();
        let hatcher = g.add_card_to_battlefield(0, catalog::drake_hatcher());
        g.battlefield_find_mut(hatcher).unwrap().add_counters(CounterType::Incubation, 3);
        g.clear_sickness(hatcher);
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        let drakes = |g: &GameState| g.battlefield.iter().filter(|c| c.controller == 0 && c.definition.name == "Drake").count();
        assert_eq!(drakes(&g), 0);
        g.perform_action(GameAction::ActivateAbility {
            card_id: hatcher, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
        }).expect("remove three incubation counters");
        drain_stack(&mut g);
        assert_eq!(g.battlefield_find(hatcher).unwrap().counter_count(CounterType::Incubation), 0, "counters spent");
        assert_eq!(drakes(&g), 1, "a Drake token entered");
    }

    /// Myojin of Night's Reach enters with a divinity counter when cast and is
    /// indestructible while it has one.
    #[test]
    fn myojin_divinity_grants_indestructible() {
        let mut g = two_player_game();
        let myojin = catalog::myojin_of_nights_reach();
        let effect = myojin.triggered_abilities[0].effect.clone();
        let id = g.add_card_to_battlefield(0, catalog::myojin_of_nights_reach());
        g.battlefield_find_mut(id).unwrap().entered_by_cast = true;
        let ctx = EffectContext::for_trigger(id, 0, None, 0);
        g.resolve_effect(&effect, &ctx).unwrap();
        assert_eq!(g.battlefield_find(id).unwrap().counter_count(CounterType::Divinity), 1, "divinity counter");
        let v = g.computed_permanent(id).unwrap();
        assert!(v.keywords.contains(&crabomination::card::Keyword::Indestructible), "indestructible while divinity present");
    }

    /// Myojin's activated ability empties each opponent's hand.
    #[test]
    fn myojin_ability_empties_opponent_hand() {
        let mut g = two_player_game();
        let id = g.add_card_to_battlefield(0, catalog::myojin_of_nights_reach());
        g.battlefield_find_mut(id).unwrap().add_counters(CounterType::Divinity, 1);
        for _ in 0..3 { g.add_card_to_hand(1, catalog::grizzly_bears()); }
        g.clear_sickness(id);
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::ActivateAbility {
            card_id: id, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
        }).expect("remove divinity: each opponent discards their hand");
        drain_stack(&mut g);
        assert_eq!(g.players[1].hand.len(), 0, "opponent hand emptied");
    }
}

mod recent220 {
    use crabomination::card::{CardType, CounterType, Keyword};
    use crabomination::catalog;
    use crabomination::effect::{Effect, Selector, Value};
    use crabomination::game::effects::EffectContext;
    use crabomination::game::types::{GameAction, Target, TurnStep};
    use crabomination::game::{drain_stack, two_player_game};

    /// Stocking the Pantry gains a supply counter when you counter-up a creature,
    /// and spends it to draw.
    #[test]
    fn stocking_the_pantry_banks_and_draws() {
        let mut g = two_player_game();
        let pantry = g.add_card_to_battlefield(0, catalog::stocking_the_pantry());
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.add_card_to_library(0, catalog::forest());
        // Putting a +1/+1 counter on my creature banks a supply counter.
        let add = Effect::AddCounter {
            what: Selector::Target(0),
            kind: CounterType::PlusOnePlusOne,
            amount: Value::ONE,
        };
        let ctx = EffectContext { targets: vec![Target::Permanent(bear)], ..EffectContext::for_trigger(bear, 0, None, 0) };
        let evs = g.resolve_effect(&add, &ctx).unwrap();
        g.dispatch_triggers_for_events(&evs);
        drain_stack(&mut g);
        assert_eq!(g.battlefield_find(pantry).unwrap().counter_count(CounterType::Supply), 1, "banked a supply counter");

        // Spend it: {2}, remove a supply counter: draw.
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g.players[0].mana_pool.add_colorless(2);
        let hand = g.players[0].hand.len();
        g.perform_action(GameAction::ActivateAbility {
            card_id: pantry, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
        }).expect("remove a supply counter to draw");
        drain_stack(&mut g);
        assert_eq!(g.players[0].hand.len(), hand + 1, "drew a card");
        assert_eq!(g.battlefield_find(pantry).unwrap().counter_count(CounterType::Supply), 0, "supply counter spent");
    }

    /// War Squeak's ETB makes an opponent's creature unable to block.
    #[test]
    fn war_squeak_grants_cant_block() {
        let mut g = two_player_game();
        let blocker = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let squeak = g.add_card_to_battlefield(0, catalog::war_squeak());
        let effect = catalog::war_squeak().triggered_abilities[0].effect.clone();
        let ctx = EffectContext { targets: vec![Target::Permanent(blocker)], ..EffectContext::for_trigger(squeak, 0, None, 0) };
        g.resolve_effect(&effect, &ctx).unwrap();
        assert!(g.computed_permanent(blocker).unwrap().keywords.contains(&Keyword::CantBlock), "opponent's creature can't block");
    }

    /// Tangle Tumbler animates itself by tapping two tokens.
    #[test]
    fn tangle_tumbler_animates_via_tokens() {
        let mut g = two_player_game();
        let tumbler = g.add_card_to_battlefield(0, catalog::tangle_tumbler());
        let tokens: Vec<_> = (0..2).map(|_| g.add_card_to_battlefield(0, catalog::grizzly_bears())).collect();
        for &t in &tokens { g.battlefield_find_mut(t).unwrap().is_token = true; }
        g.clear_sickness(tumbler);
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        assert!(!g.computed_permanent(tumbler).unwrap().card_types.contains(&CardType::Creature), "starts as a non-creature Vehicle");
        g.perform_action(GameAction::ActivateAbility {
            card_id: tumbler, ability_index: 1, target: None, additional_targets: Vec::new(), x_value: None,
        }).expect("tap two tokens to animate");
        drain_stack(&mut g);
        assert!(g.computed_permanent(tumbler).unwrap().card_types.contains(&CardType::Creature), "now an artifact creature");
        assert_eq!(tokens.iter().filter(|&&t| g.battlefield_find(t).unwrap().tapped).count(), 2, "two tokens tapped");
    }

    /// Bonecache Overseer only draws once three cards have left your graveyard.
    #[test]
    fn bonecache_overseer_gated_on_graveyard_departures() {
        let mut g = two_player_game();
        let overseer = g.add_card_to_battlefield(0, catalog::bonecache_overseer());
        g.add_card_to_library(0, catalog::forest());
        g.clear_sickness(overseer);
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        // Not yet: fewer than three cards have left the graveyard.
        assert!(g.perform_action(GameAction::ActivateAbility {
            card_id: overseer, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
        }).is_err(), "gated until three cards leave the graveyard");
        g.players[0].cards_left_graveyard_this_turn = 3;
        let hand = g.players[0].hand.len();
        g.perform_action(GameAction::ActivateAbility {
            card_id: overseer, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
        }).expect("now activatable");
        drain_stack(&mut g);
        assert_eq!(g.players[0].hand.len(), hand + 1, "drew a card");
    }
}

mod recent221 {
    use crabomination::card::{CardType, Supertype};
    use crabomination::catalog;
    use crabomination::game::types::TurnStep;
    use crabomination::game::{drain_stack, two_player_game, GameAction};
    use crabomination::mana::Color;

    /// Diamond Mare gains life only when you cast a spell of its chosen color.
    #[test]
    fn diamond_mare_gains_on_chosen_color() {
        let mut g = two_player_game();
        let mare = g.add_card_to_battlefield(0, catalog::diamond_mare());
        g.battlefield_find_mut(mare).unwrap().chosen_color = Some(Color::Red);
        // A red spell to cast.
        let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g.players[0].mana_pool.add(Color::Red, 1);
        let life = g.players[0].life;
        g.perform_action(GameAction::CastSpell {
            card_id: bolt, target: Some(crabomination::game::types::Target::Player(1)), additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast red spell");
        drain_stack(&mut g);
        assert_eq!(g.players[0].life, life + 1, "gained 1 life for the red spell");
    }

    /// The Aetherdrift Tyrant legends have their printed stats and supertype.
    #[test]
    fn tyrant_legends_have_printed_stats() {
        type Make = fn() -> crabomination::card::CardDefinition;
        for (make, pt) in [
            (catalog::kalakscion_hunger_tyrant as Make, (7, 2)),
            (catalog::tyrox_saurid_tyrant as Make, (4, 1)),
            (catalog::terrian_world_tyrant as Make, (9, 7)),
            (catalog::sundial_dawn_tyrant as Make, (3, 3)),
        ] {
            let def = make();
            assert!(def.supertypes.contains(&Supertype::Legendary), "{} is legendary", def.name);
            assert!(def.card_types.contains(&CardType::Creature), "{} is a creature", def.name);
            assert_eq!((def.power, def.toughness), pt, "{} stats", def.name);
        }
        assert!(catalog::sundial_dawn_tyrant().card_types.contains(&CardType::Artifact), "Sundial is an artifact");
    }
}

mod recent222 {
    use crabomination::catalog;
    use crabomination::game::types::{GameAction, TurnStep};
    use crabomination::game::{drain_stack, two_player_game};
    use crabomination::mana::Color;

    /// Vizier of the Menagerie lets you cast a creature spell off the top of your
    /// library.
    #[test]
    fn vizier_casts_creature_from_top() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::vizier_of_the_menagerie());
        let bears = g.add_card_to_library(0, catalog::grizzly_bears()); // top of library
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g.players[0].mana_pool.add(Color::Green, 1);
        g.players[0].mana_pool.add_colorless(1);
        g.perform_action(GameAction::CastSpell {
            card_id: bears, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast Grizzly Bears from the top of the library");
        drain_stack(&mut g);
        assert!(g.battlefield.iter().any(|c| c.id == bears && c.controller == 0), "Grizzly Bears entered from the top");
    }
}
