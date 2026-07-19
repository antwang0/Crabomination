//! Tests for recentN card batches 193-207 (merged from per-batch micro-files).

mod recent193 {
    use crabomination::catalog;
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    use crabomination::game::*;
    use crabomination::game::{drain_stack, two_player_game};
    use crabomination::mana::Color;

    /// Jackdaw Savior: when another flying creature you control dies, reanimate a
    /// lesser-mana-value creature card from your graveyard.
    #[test]
    fn jackdaw_savior_reanimates_lesser_mv() {
        let mut g = two_player_game();
        g.step = TurnStep::PreCombatMain;
        g.add_card_to_battlefield(0, catalog::jackdaw_savior()); // surviving 3/1 flyer
        let victim = g.add_card_to_battlefield(0, catalog::jackdaw_savior()); // MV 3 flyer dies
        let target = g.add_card_to_graveyard(0, catalog::grizzly_bears()); // MV 2 < 3
        g.battlefield_find_mut(victim).unwrap().damage = 1; // lethal on the 3/1
        let evs = g.check_state_based_actions();
        g.dispatch_triggers_for_events(&evs);
        drain_stack(&mut g);
        assert!(g.battlefield_find(target).is_some(), "grizzly returned to the battlefield");
    }

    /// Clement's enter trigger bounces a lesser-mana-value creature you control.
    #[test]
    fn clement_bounces_lesser_mv_on_enter() {
        let mut g = two_player_game();
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // MV 2
        let clement = g.add_card_to_hand(0, catalog::clement_the_worrywort()); // MV 3
        g.players[0].mana_pool.add(Color::Green, 1);
        g.players[0].mana_pool.add(Color::Blue, 1);
        g.players[0].mana_pool.add_colorless(1);
        g.perform_action(GameAction::CastSpell {
            card_id: clement,
            target: None,
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .expect("cast Clement");
        drain_stack(&mut g);
        assert!(g.players[0].hand.iter().any(|c| c.id == bear), "lesser-MV bear bounced to hand");
    }

    /// Soul-Shackled Zombie: exiling a creature card from a graveyard drains the
    /// opponent for 2 and gains you 2.
    #[test]
    fn soul_shackled_zombie_creature_exile_drains() {
        let mut g = two_player_game();
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        let victim = g.add_card_to_graveyard(1, catalog::grizzly_bears()); // creature card
        g.players[0].life = 20;
        g.players[1].life = 20;
        let zombie = g.add_card_to_hand(0, catalog::soul_shackled_zombie());
        g.players[0].mana_pool.add(Color::Black, 1);
        g.players[0].mana_pool.add_colorless(3);
        // Supply the exile pick (auto-decider declines an "up to N" choice).
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Cards(vec![victim])]));
        g.perform_action(GameAction::CastSpell {
            card_id: zombie,
            target: None,
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .expect("cast Soul-Shackled Zombie");
        drain_stack(&mut g);
        assert!(g.exile.iter().any(|c| c.id == victim), "creature card exiled");
        assert_eq!(g.players[1].life, 18, "opponent lost 2");
        assert_eq!(g.players[0].life, 22, "you gained 2");
    }
}

mod recent194 {
    use crabomination::catalog;
    
    use crabomination::game::*;
    use crabomination::game::{drain_stack, two_player_game};
    use crabomination::mana::Color;

    /// Double Down copies an outlaw (Rogue) spell you cast.
    #[test]
    fn double_down_copies_outlaw_spell() {
        let mut g = two_player_game();
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g.add_card_to_battlefield(0, catalog::double_down());
        // Grizzly Bears is a Bear (not outlaw) → no copy; use a Rogue creature spell.
        let rogue = g.add_card_to_hand(0, catalog::servant_of_the_stinger()); // Warlock = outlaw
        g.players[0].mana_pool.add(Color::Black, 1);
        g.players[0].mana_pool.add_colorless(1);
        g.perform_action(GameAction::CastSpell {
            card_id: rogue,
            target: None,
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .expect("cast outlaw spell");
        drain_stack(&mut g);
        let servants = g
            .battlefield
            .iter()
            .filter(|c| c.definition.name == "Servant of the Stinger" && c.controller == 0)
            .count();
        assert_eq!(servants, 2, "outlaw spell copied → two Servants (copy is a token)");
    }

    /// Mystical Tether exiles an opponent's creature until it leaves.
    #[test]
    fn mystical_tether_exiles_until_leaves() {
        let mut g = two_player_game();
        g.step = TurnStep::PreCombatMain;
        let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let tether = g.add_card_to_battlefield(0, catalog::mystical_tether());
        g.fire_self_etb_triggers(tether, 0);
        drain_stack(&mut g);
        assert!(g.battlefield_find(victim).is_none(), "opponent's creature exiled");
        // Destroy the Tether → the creature returns.
        let ctx = crabomination::game::effects::EffectContext::for_ability(tether, 0, None);
        let evs = g
            .resolve_effect(&crabomination::effect::Effect::Destroy { what: crabomination::effect::Selector::This }, &ctx)
            .unwrap();
        g.dispatch_triggers_for_events(&evs);
        drain_stack(&mut g);
        assert!(
            g.battlefield.iter().any(|c| c.definition.name == "Grizzly Bears" && c.controller == 1),
            "creature returned when the Tether left",
        );
    }

    /// High Noon bars a second spell each turn.
    #[test]
    fn high_noon_one_spell_per_turn() {
        let mut g = two_player_game();
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g.add_card_to_battlefield(0, catalog::high_noon());
        g.players[0].spells_cast_this_game_turn = 1; // already cast one this turn
        let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
        g.players[0].mana_pool.add(Color::Green, 2);
        let res = g.perform_action(GameAction::CastSpell {
            card_id: bear,
            target: None,
            additional_targets: vec![],
            mode: None,
            x_value: None,
        });
        assert!(res.is_err(), "second spell barred by High Noon");
    }
}

mod recent195 {
    use crabomination::card::Keyword;
    use crabomination::catalog;
    use crabomination::game::types::Target;
    use crabomination::game::*;
    use crabomination::game::{drain_stack, two_player_game};
    use crabomination::mana::Color;

    /// Malcolm investigates on your second spell each turn.
    #[test]
    fn malcolm_investigates_on_second_spell() {
        let mut g = two_player_game();
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g.add_card_to_battlefield(0, catalog::malcolm_the_eyes());
        // Cast two cheap spells; the second should mint a Clue.
        for _ in 0..2 {
            let b = g.add_card_to_hand(0, catalog::lightning_bolt());
            g.players[0].mana_pool.add(Color::Red, 1);
            g.perform_action(GameAction::CastSpell {
                card_id: b, target: Some(Target::Player(1)),
                additional_targets: vec![], mode: None, x_value: None,
            }).expect("cast bolt");
            drain_stack(&mut g);
        }
        assert!(g.battlefield.iter().any(|c| c.definition.name == "Clue"), "second spell investigated");
    }

    /// Reach for the Sky pumps and draws when it dies.
    #[test]
    fn reach_for_the_sky_pumps_and_draws_on_death() {
        let mut g = two_player_game();
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
        let aura = g.add_card_to_hand(0, catalog::reach_for_the_sky());
        g.players[0].mana_pool.add(Color::Green, 1);
        g.players[0].mana_pool.add_colorless(3);
        g.add_card_to_library(0, catalog::forest());
        let hand_before = g.players[0].hand.len();
        g.perform_action(GameAction::CastSpell {
            card_id: aura, target: Some(Target::Permanent(bear)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast Reach for the Sky");
        drain_stack(&mut g);
        let c = g.compute_battlefield();
        let c = c.iter().find(|c| c.id == bear).unwrap();
        assert_eq!((c.power, c.toughness), (5, 4), "+3/+2");
        assert!(c.keywords.contains(&Keyword::Reach), "granted reach");
        // Destroy the host → Aura goes to graveyard → draw.
        let ctx = crabomination::game::effects::EffectContext::for_ability(bear, 0, None);
        let evs = g.resolve_effect(&crabomination::effect::Effect::Destroy { what: crabomination::effect::Selector::This }, &ctx).unwrap();
        g.dispatch_triggers_for_events(&evs);
        drain_stack(&mut g);
        assert_eq!(g.players[0].hand.len(), hand_before, "drew a card when the Aura died");
    }

    /// Tomb Trawler tucks a graveyard card to the bottom of the library.
    #[test]
    fn tomb_trawler_tucks_graveyard_card() {
        let mut g = two_player_game();
        let trawler = g.add_card_to_battlefield(0, catalog::tomb_trawler());
        g.clear_sickness(trawler);
        let bolt = g.add_card_to_graveyard(0, catalog::lightning_bolt());
        g.players[0].mana_pool.add_colorless(2);
        g.perform_action(GameAction::ActivateAbility {
            card_id: trawler, ability_index: 0, target: Some(Target::Permanent(bolt)),
            additional_targets: vec![], x_value: None,
        }).expect("tuck");
        drain_stack(&mut g);
        assert_eq!(g.players[0].library.last().map(|c| c.id), Some(bolt), "bolt on the bottom");
    }

    /// Steer Clear scales to 4 damage while you control a Mount.
    #[test]
    fn steer_clear_mount_scales_damage() {
        let mut g = two_player_game();
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        // An attacking 4-toughness creature; 2 wouldn't kill it, 4 does.
        let victim = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4
        g.attacking.push(crabomination::game::types::Attack {
            attacker: victim,
            target: crabomination::game::types::AttackTarget::Player(0),
        });
        g.add_card_to_battlefield(0, catalog::drover_grizzly());
        let spell = g.add_card_to_hand(0, catalog::steer_clear());
        g.players[0].mana_pool.add(Color::White, 1);
        g.perform_action(GameAction::CastSpell {
            card_id: spell, target: Some(Target::Permanent(victim)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast Steer Clear");
        drain_stack(&mut g);
        assert_eq!(g.battlefield_find(victim).map(|c| c.damage), None, "4 damage with a Mount killed the 4/4");
    }
}

mod recent196 {
    use crabomination::card::CounterType;
    use crabomination::catalog;
    use crabomination::game::types::Target;
    use crabomination::game::*;
    use crabomination::game::{drain_stack, two_player_game};
    use crabomination::mana::Color;

    /// Slickshot Vault-Buster is a 1/4 that swings to 3/4 after a crime.
    #[test]
    fn slickshot_vault_buster_crime_pump() {
        let mut g = two_player_game();
        let id = g.add_card_to_battlefield(0, catalog::slickshot_vault_buster());
        assert_eq!(g.computed_permanent(id).unwrap().power, 1, "no crime → 1/4");
        g.players[0].committed_crime_this_turn = true;
        assert_eq!(g.computed_permanent(id).unwrap().power, 3, "crime → +2/+0");
    }

    /// Throw from the Saddle counters a Mount, then it deals its (boosted) power.
    #[test]
    fn throw_from_the_saddle_mount_counter_and_fight() {
        let mut g = two_player_game();
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        let mount = g.add_card_to_battlefield(0, catalog::drover_grizzly()); // 2/2 Bear Mount
        let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
        let spell = g.add_card_to_hand(0, catalog::throw_from_the_saddle());
        g.players[0].mana_pool.add(Color::Green, 1);
        g.players[0].mana_pool.add_colorless(1);
        g.perform_action(GameAction::CastSpell {
            card_id: spell,
            target: Some(Target::Permanent(mount)),
            additional_targets: vec![Target::Permanent(foe)],
            mode: None,
            x_value: None,
        })
        .expect("cast Throw from the Saddle");
        drain_stack(&mut g);
        assert_eq!(
            g.battlefield_find(mount).unwrap().counter_count(CounterType::PlusOnePlusOne),
            1,
            "Mount got a +1/+1 counter",
        );
        assert!(g.battlefield_find(foe).is_none(), "3 power killed the 2/2");
    }

    /// Shepherd returns a graveyard permanent to hand with no Mount, straight to the
    /// battlefield with one.
    #[test]
    fn shepherd_of_the_clouds_mount_upgrade() {
        // No Mount → returns to hand.
        let mut g = two_player_game();
        let target = g.add_card_to_graveyard(0, catalog::grizzly_bears()); // MV 2
        let shep = g.add_card_to_battlefield(0, catalog::shepherd_of_the_clouds());
        g.fire_self_etb_triggers(shep, 0);
        drain_stack(&mut g);
        assert!(g.players[0].hand.iter().any(|c| c.id == target), "returned to hand");

        // With a Mount → returns to the battlefield.
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::drover_grizzly()); // Mount
        let target = g.add_card_to_graveyard(0, catalog::grizzly_bears());
        let shep = g.add_card_to_battlefield(0, catalog::shepherd_of_the_clouds());
        g.fire_self_etb_triggers(shep, 0);
        drain_stack(&mut g);
        assert!(g.battlefield_find(target).is_some(), "returned to the battlefield with a Mount");
    }

    /// Sheriff enters with 1 counter plus one per other creature you control.
    #[test]
    fn sheriff_scales_with_your_board() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let sheriff = g.move_card_to_battlefield_for_test(0, catalog::sheriff_of_safe_passage());
        // 1 base + 2 other creatures = 3 counters → 3/3.
        assert_eq!(
            g.battlefield_find(sheriff).unwrap().counter_count(CounterType::PlusOnePlusOne),
            3,
            "1 + other creatures",
        );
    }
}

mod recent197 {
    use crabomination::catalog;
    use crabomination::game::types::Target;
    use crabomination::game::*;
    use crabomination::game::{drain_stack, two_player_game};
    use crabomination::mana::Color;

    /// Seize the Secrets costs {1} less after a crime.
    #[test]
    fn seize_the_secrets_crime_discount() {
        let mut g = two_player_game();
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g.players[0].committed_crime_this_turn = true;
        for _ in 0..3 {
            g.add_card_to_library(0, catalog::grizzly_bears());
        }
        let spell = g.add_card_to_hand(0, catalog::seize_the_secrets());
        // Only {1}{U} available — the crime discount must apply for this to cast.
        g.players[0].mana_pool.add(Color::Blue, 1);
        g.players[0].mana_pool.add_colorless(1);
        let hand_before = g.players[0].hand.len();
        g.perform_action(GameAction::CastSpell {
            card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None,
        })
        .expect("crime discount makes it {1}{U}");
        drain_stack(&mut g);
        // Cast one card out of hand, drew two → net +1.
        assert_eq!(g.players[0].hand.len(), hand_before - 1 + 2, "drew two cards");
    }

    /// Take for a Ride steals a creature, untapping it and granting haste.
    #[test]
    fn take_for_a_ride_steals_and_hastes() {
        let mut g = two_player_game();
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        g.battlefield_find_mut(victim).unwrap().tapped = true;
        let spell = g.add_card_to_hand(0, catalog::take_for_a_ride());
        g.players[0].mana_pool.add(Color::Red, 1);
        g.players[0].mana_pool.add_colorless(2);
        g.perform_action(GameAction::CastSpell {
            card_id: spell, target: Some(Target::Permanent(victim)),
            additional_targets: vec![], mode: None, x_value: None,
        })
        .expect("cast Take for a Ride");
        drain_stack(&mut g);
        let v = g.battlefield_find(victim).unwrap();
        assert_eq!(v.controller, 0, "gained control");
        assert!(!v.tapped, "untapped");
        assert!(g.compute_battlefield().iter().find(|c| c.id == victim).unwrap()
            .keywords.contains(&crabomination::card::Keyword::Haste), "granted haste");
    }

    /// Silver Deputy digs a basic to the top of your library on ETB.
    #[test]
    fn silver_deputy_digs_a_basic() {
        use crabomination::decision::{DecisionAnswer, ScriptedDecider};
        let mut g = two_player_game();
        g.add_card_to_library(0, catalog::grizzly_bears()); // non-land noise
        let forest_id = g.add_card_to_library(0, catalog::forest());
        let dep = g.add_card_to_battlefield(0, catalog::silver_deputy());
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(forest_id))]));
        g.fire_self_etb_triggers(dep, 0);
        drain_stack(&mut g);
        assert_eq!(g.players[0].library.first().map(|c| c.id), Some(forest_id), "basic on top");
    }
}

mod recent198 {
    use crabomination::catalog;
    use crabomination::game::types::{Attack, AttackTarget, Target};
    use crabomination::game::*;
    use crabomination::game::{drain_stack, two_player_game};

    /// Baseball Bat auto-attaches on ETB (+1/+1) and taps a creature when its
    /// wielder attacks.
    #[test]
    fn baseball_bat_attaches_and_taps_on_attack() {
        let mut g = two_player_game();
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        let bearer = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2 → 3/3
        let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let bat = g.add_card_to_battlefield(0, catalog::baseball_bat());
        g.decider = Box::new(crabomination::decision::ScriptedDecider::new([
            crabomination::decision::DecisionAnswer::Target(Target::Permanent(bearer)),
        ]));
        g.fire_self_etb_triggers(bat, 0);
        drain_stack(&mut g);
        assert_eq!(g.battlefield_find(bat).unwrap().attached_to, Some(bearer), "attached to the bearer");
        assert_eq!(g.computed_permanent(bearer).unwrap().power, 3, "+1/+1 from the Bat");

        // Attack with the bearer → tap the opposing creature.
        g.clear_sickness(bearer);
        g.step = TurnStep::DeclareAttackers;
        g.decider = Box::new(crabomination::decision::ScriptedDecider::new([
            crabomination::decision::DecisionAnswer::Target(Target::Permanent(foe)),
        ]));
        g.declare_attackers(vec![Attack { attacker: bearer, target: AttackTarget::Player(1) }])
            .expect("attack");
        drain_stack(&mut g);
        assert!(g.battlefield_find(foe).unwrap().tapped, "attack trigger tapped the foe");
    }
}

mod recent199 {
    
    use crabomination::catalog;
    use crabomination::game::*;
    use crabomination::game::{drain_stack, two_player_game};

    /// Growing Dread manifests dread on ETB (a face-down 2/2 appears).
    #[test]
    fn growing_dread_manifests_dread() {
        let mut g = two_player_game();
        g.add_card_to_library(0, catalog::grizzly_bears());
        g.add_card_to_library(0, catalog::forest());
        let gd = g.add_card_to_battlefield(0, catalog::growing_dread());
        g.fire_self_etb_triggers(gd, 0);
        drain_stack(&mut g);
        assert!(
            g.battlefield.iter().any(|c| c.controller == 0 && c.face_down),
            "a face-down manifest entered",
        );
    }

    /// Entity Tracker draws when another enchantment you control enters.
    #[test]
    fn entity_tracker_draws_on_enchantment() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::entity_tracker());
        g.add_card_to_library(0, catalog::grizzly_bears());
        let hand_before = g.players[0].hand.len();
        let ench = g.add_card_to_battlefield(0, catalog::growing_dread()); // an enchantment enters
        g.dispatch_triggers_for_events(&[GameEvent::PermanentEntered { card_id: ench }]);
        drain_stack(&mut g);
        assert_eq!(g.players[0].hand.len(), hand_before + 1, "Eerie drew a card");
    }
}

mod recent200 {
    use crabomination::card::Keyword;
    use crabomination::catalog;
    use crabomination::game::*;
    use crabomination::game::{drain_stack, two_player_game};

    /// Dragonfire Blade grants +2/+2 and hexproof from monocolored to its bearer.
    #[test]
    fn dragonfire_blade_equips_and_buffs() {
        let mut g = two_player_game();
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        let bearer = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
        let blade = g.add_card_to_battlefield(0, catalog::dragonfire_blade());
        g.players[0].mana_pool.add_colorless(4);
        g.perform_action(GameAction::Equip { equipment: blade, target: bearer })
            .expect("equip Dragonfire Blade");
        drain_stack(&mut g);
        let cp = g.computed_permanent(bearer).unwrap();
        assert_eq!((cp.power, cp.toughness), (4, 4), "+2/+2 from the Blade");
        assert!(cp.keywords.contains(&Keyword::HexproofFromMonocolored), "granted hexproof from monocolored");
    }
}

mod recent201 {
    use crabomination::catalog;
    use crabomination::game::types::Target;
    use crabomination::game::*;
    use crabomination::game::{drain_stack, two_player_game};
    use crabomination::mana::Color;

    /// Duskmourn's Domination steals a creature and shrinks + silences it.
    #[test]
    fn duskmourns_domination_steals_and_shrinks() {
        let mut g = two_player_game();
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        // A 4/4 flyer to steal; -3/-0 leaves a 1/4 you control, no flying.
        let victim = g.add_card_to_battlefield(1, catalog::serra_angel());
        let aura = g.add_card_to_hand(0, catalog::duskmourns_domination());
        g.players[0].mana_pool.add(Color::Blue, 2);
        g.players[0].mana_pool.add_colorless(4);
        g.perform_action(GameAction::CastSpell {
            card_id: aura, target: Some(Target::Permanent(victim)),
            additional_targets: vec![], mode: None, x_value: None,
        })
        .expect("cast Duskmourn's Domination");
        drain_stack(&mut g);
        let cp = g.computed_permanent(victim).unwrap();
        assert_eq!(cp.controller, 0, "you control the enchanted creature");
        assert_eq!(cp.power, 1, "-3/-0 leaves 1 power");
        assert!(!cp.keywords.contains(&crabomination::card::Keyword::Flying), "lost its abilities");
    }
}

mod recent202 {
    use crabomination::catalog;
    use crabomination::game::types::Target;
    use crabomination::game::*;
    use crabomination::game::{drain_stack, two_player_game};
    use crabomination::mana::Color;

    /// Rite of the Dragoncaller mints a 5/5 flying Dragon on each instant/sorcery cast.
    #[test]
    fn rite_of_the_dragoncaller_mints_dragon_on_spell() {
        let mut g = two_player_game();
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g.add_card_to_battlefield(0, catalog::rite_of_the_dragoncaller());
        let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
        g.players[0].mana_pool.add(Color::Red, 1);
        let before = g.battlefield.iter().filter(|c| c.controller == 0 && c.definition.is_creature()).count();
        g.perform_action(GameAction::CastSpell {
            card_id: bolt, target: Some(Target::Player(1)),
            additional_targets: vec![], mode: None, x_value: None,
        })
        .expect("cast Lightning Bolt");
        drain_stack(&mut g);
        let dragons = g.battlefield.iter().filter(|c| {
            c.controller == 0 && c.definition.name == "Dragon"
        }).count();
        assert_eq!(dragons, 1, "one 5/5 Dragon token minted");
        let after = g.battlefield.iter().filter(|c| c.controller == 0 && c.definition.is_creature()).count();
        assert_eq!(after, before + 1);
    }

    /// Koma makes four Koma's Coil tokens on combat damage to a player.
    #[test]
    fn koma_makes_four_coils_on_combat_damage() {
        let mut g = two_player_game();
        let koma = g.add_card_to_battlefield(0, catalog::koma_world_eater());
        g.fire_combat_damage_to_player_triggers(koma, 1, 8);
        drain_stack(&mut g);
        let coils = g.battlefield.iter().filter(|c| c.definition.name == "Koma's Coil").count();
        assert_eq!(coils, 4, "four 3/3 Serpent coils");
    }

    /// Koma can't be countered and has ward {4}.
    #[test]
    fn koma_is_uncounterable_and_warded() {
        use crabomination::card::{Keyword, WardCost};
        let k = catalog::koma_world_eater();
        assert!(k.keywords.contains(&Keyword::CantBeCountered));
        assert!(k.keywords.iter().any(|kw| matches!(kw, Keyword::Ward(WardCost::Mana(_)))));
    }

    /// Niv-Mizzet draws for noncombat damage its controller's sources deal to an
    /// opponent, and grants no maximum hand size.
    #[test]
    fn niv_mizzet_draws_on_noncombat_damage() {
        let mut g = two_player_game();
        let niv = g.add_card_to_battlefield(0, catalog::niv_mizzet_visionary());
        for _ in 0..5 { g.add_card_to_library(0, catalog::grizzly_bears()); }
        let hand0 = g.players[0].hand.len();
        let _ = niv;
        let evs = vec![GameEvent::DamageDealt {
            amount: 3,
            to_player: Some(1),
            to_card: None,
            from_controller: Some(0),
            combat: false,
        }];
        g.dispatch_triggers_for_events(&evs);
        drain_stack(&mut g);
        assert_eq!(g.players[0].hand.len(), hand0 + 3, "drew 3 for 3 noncombat damage");
        assert!(g.effective_max_hand_size(0).is_none(), "no maximum hand size");
    }

    /// Perforating Artist's Raid punisher only fires if you attacked; the opponent
    /// loses 3 life when they can't/won't pay.
    #[test]
    fn perforating_artist_raid_punisher() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::perforating_artist());
        g.active_player_idx = 0;
        // Mark that player 0 attacked this turn (Raid). Empty the opponent's hand
        // and board so they can't shed via discard/sacrifice.
        g.players[0].attacked_this_turn = true;
        g.players[1].hand.clear();
        let l1 = g.players[1].life;
        g.fire_step_triggers(TurnStep::End);
        drain_stack(&mut g);
        assert_eq!(g.players[1].life, l1 - 3, "opponent lost 3 (no nonland/discard to shed)");
    }

    /// Kiora's ETB draws two and discards two.
    #[test]
    fn kiora_etb_loots_two() {
        let mut g = two_player_game();
        for _ in 0..4 { g.add_card_to_library(0, catalog::grizzly_bears()); }
        let hand0 = g.players[0].hand.len();
        let kiora = g.add_card_to_battlefield(0, catalog::kiora_the_rising_tide());
        g.fire_self_etb_triggers(kiora, 0);
        drain_stack(&mut g);
        assert_eq!(g.players[0].hand.len(), hand0, "drew two, discarded two");
        assert_eq!(g.players[0].graveyard.len(), 2, "two cards discarded to graveyard");
    }

    /// Kiora's threshold attack trigger makes an 8/8 Scion once seven cards are in
    /// the graveyard.
    #[test]
    fn kiora_threshold_makes_scion() {
        use crabomination::decision::{DecisionAnswer, ScriptedDecider};
        use crabomination::game::types::{Attack, AttackTarget};
        let mut g = two_player_game();
        for _ in 0..7 { g.add_card_to_graveyard(0, catalog::grizzly_bears()); }
        let kiora = g.add_card_to_battlefield(0, catalog::kiora_the_rising_tide());
        g.clear_sickness(kiora);
        g.step = TurnStep::DeclareAttackers;
        g.priority.player_with_priority = 0;
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
        let evs = g
            .declare_attackers(vec![Attack { attacker: kiora, target: AttackTarget::Player(1) }])
            .expect("Kiora attacks");
        g.dispatch_triggers_for_events(&evs);
        drain_stack(&mut g);
        let scions = g.battlefield.iter().filter(|c| c.definition.name == "Scion of the Deep").count();
        assert_eq!(scions, 1, "8/8 Scion minted at threshold");
    }

    /// Lunar Insight draws once per distinct mana value among your nonland permanents.
    #[test]
    fn lunar_insight_draws_per_distinct_mv() {
        let mut g = two_player_game();
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        // Distinct MVs among nonland permanents: Grizzly Bears (2) and Lightning
        // Bolt isn't a permanent — use two creatures of different MV.
        g.add_card_to_battlefield(0, catalog::grizzly_bears()); // MV 2
        g.add_card_to_battlefield(0, catalog::grizzly_bears()); // MV 2 (dup)
        g.add_card_to_battlefield(0, catalog::serra_angel()); // MV 5
        for _ in 0..5 { g.add_card_to_library(0, catalog::grizzly_bears()); }
        let li = g.add_card_to_hand(0, catalog::lunar_insight());
        g.players[0].mana_pool.add(Color::Blue, 1);
        g.players[0].mana_pool.add_colorless(2);
        let hand0 = g.players[0].hand.len();
        g.perform_action(GameAction::CastSpell {
            card_id: li, target: None, additional_targets: vec![], mode: None, x_value: None,
        })
        .expect("cast Lunar Insight");
        drain_stack(&mut g);
        // Two distinct MVs (2 and 5) → draw 2. Hand was hand0, minus the cast spell.
        assert_eq!(g.players[0].hand.len(), hand0 - 1 + 2, "drew 2 for 2 distinct MVs");
    }

    /// Soulstone Sanctuary animates into a 3/3 with all creature types (Changeling).
    #[test]
    fn soulstone_sanctuary_animates_all_types() {
        let mut g = two_player_game();
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        let land = g.add_card_to_battlefield(0, catalog::soulstone_sanctuary());
        g.players[0].mana_pool.add_colorless(4);
        // Activate the {4} animate ability (index 1).
        g.perform_action(GameAction::ActivateAbility {
            card_id: land, ability_index: 1, target: None, additional_targets: vec![],
            x_value: None,
        })
        .expect("animate Soulstone Sanctuary");
        drain_stack(&mut g);
        let c = g.computed_permanent(land).expect("soulstone");
        assert!(c.card_types.contains(&crabomination::card::CardType::Creature), "now a creature");
        assert!(c.card_types.contains(&crabomination::card::CardType::Land), "still a land");
        assert_eq!((c.power, c.toughness), (3, 3));
        assert!(c.keywords.contains(&crabomination::card::Keyword::Changeling), "all creature types via Changeling");
    }
}

mod recent203 {
    use crabomination::card::{CounterType, CreatureType, Keyword};
    use crabomination::catalog;
    use crabomination::game::types::Target;
    use crabomination::game::*;
    use crabomination::game::{drain_stack, two_player_game};
    use crabomination::mana::Color;

    /// Valkyrie's Call returns a slain nontoken non-Angel with a counter, flying, and
    /// the Angel type.
    #[test]
    fn valkyries_call_returns_as_angel() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::valkyries_call());
        let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.battlefield_find_mut(bears).unwrap().damage = 2; // lethal on the 2/2
        let evs = g.check_state_based_actions();
        g.dispatch_triggers_for_events(&evs);
        drain_stack(&mut g);
        let returned = g
            .battlefield
            .iter()
            .find(|c| c.definition.name == "Grizzly Bears")
            .expect("bears returned");
        assert!(returned.counter_count(CounterType::PlusOnePlusOne) >= 1, "+1/+1 counter");
        let c = g.computed_permanent(returned.id).unwrap();
        assert!(c.keywords.contains(&Keyword::Flying), "gained flying");
        assert!(c.subtypes.creature_types.contains(&CreatureType::Angel), "became an Angel");
    }

    /// Infernal Vessel returns once with two counters as a Demon, then stays dead.
    #[test]
    fn infernal_vessel_returns_as_demon_once() {
        let mut g = two_player_game();
        let iv = g.add_card_to_battlefield(0, catalog::infernal_vessel());
        g.battlefield_find_mut(iv).unwrap().damage = 1; // lethal on the 2/1
        let evs = g.check_state_based_actions();
        g.dispatch_triggers_for_events(&evs);
        drain_stack(&mut g);
        let back = g
            .battlefield
            .iter()
            .find(|c| c.definition.name == "Infernal Vessel")
            .expect("returned");
        assert_eq!(back.counter_count(CounterType::PlusOnePlusOne), 2, "two +1/+1 counters");
        let back_id = back.id;
        assert!(
            g.computed_permanent(back_id).unwrap().subtypes.creature_types.contains(&CreatureType::Demon),
            "now a Demon"
        );
        // Kill the Demon copy — it must not loop back.
        let count_before = g.battlefield.iter().filter(|c| c.definition.name == "Infernal Vessel").count();
        g.battlefield_find_mut(back_id).unwrap().damage = 99;
        let evs = g.check_state_based_actions();
        g.dispatch_triggers_for_events(&evs);
        drain_stack(&mut g);
        let count_after = g.battlefield.iter().filter(|c| c.definition.name == "Infernal Vessel").count();
        assert_eq!(count_after, count_before - 1, "the Demon copy did not return");
    }

    /// Fiery Annihilation deals 5 and exiles the creature instead of letting it die.
    #[test]
    fn fiery_annihilation_exiles_the_creature() {
        let mut g = two_player_game();
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let spell = g.add_card_to_hand(0, catalog::fiery_annihilation());
        g.players[0].mana_pool.add(Color::Red, 1);
        g.players[0].mana_pool.add_colorless(2);
        g.perform_action(GameAction::CastSpell {
            card_id: spell, target: Some(Target::Permanent(victim)),
            additional_targets: vec![], mode: None, x_value: None,
        })
        .expect("cast Fiery Annihilation");
        drain_stack(&mut g);
        let _ = g.check_state_based_actions();
        assert!(g.battlefield_find(victim).is_none(), "creature left the battlefield");
        assert!(g.exile.iter().any(|c| c.id == victim), "exiled, not in graveyard");
        assert!(!g.players[1].graveyard.iter().any(|c| c.id == victim), "not in graveyard");
    }

    /// Violent Urge grants +1/+0 and first strike; with delirium it adds double strike.
    #[test]
    fn violent_urge_first_strike_and_delirium_double_strike() {
        let mut g = two_player_game();
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        // Four card types in the graveyard → delirium active.
        g.add_card_to_graveyard(0, catalog::grizzly_bears()); // creature
        g.add_card_to_graveyard(0, catalog::lightning_bolt()); // instant
        g.add_card_to_graveyard(0, catalog::island()); // land
        g.add_card_to_graveyard(0, catalog::rite_of_the_dragoncaller()); // enchantment
        let spell = g.add_card_to_hand(0, catalog::violent_urge());
        g.players[0].mana_pool.add(Color::Red, 1);
        g.perform_action(GameAction::CastSpell {
            card_id: spell, target: Some(Target::Permanent(bear)),
            additional_targets: vec![], mode: None, x_value: None,
        })
        .expect("cast Violent Urge");
        drain_stack(&mut g);
        let c = g.computed_permanent(bear).unwrap();
        assert_eq!(c.power, 3, "+1/+0");
        assert!(c.keywords.contains(&Keyword::FirstStrike), "first strike");
        assert!(c.keywords.contains(&Keyword::DoubleStrike), "delirium double strike");
    }

    /// Elenda scales with life above her controller's starting total.
    #[test]
    fn elenda_scales_with_life() {
        let mut g = two_player_game();
        let elenda = g.add_card_to_battlefield(0, catalog::elenda_saint_of_dusk());
        let start = g.players[0].life;
        // At starting life: base 4/4, no menace.
        let c = g.computed_permanent(elenda).unwrap();
        assert_eq!((c.power, c.toughness), (4, 4));
        assert!(!c.keywords.contains(&Keyword::Menace));
        // One above: 5/5 with menace.
        g.players[0].life = start + 1;
        let c = g.computed_permanent(elenda).unwrap();
        assert_eq!((c.power, c.toughness), (5, 5));
        assert!(c.keywords.contains(&Keyword::Menace));
        // Ten above: additional +5/+5 → 10/10.
        g.players[0].life = start + 10;
        let c = g.computed_permanent(elenda).unwrap();
        assert_eq!((c.power, c.toughness), (10, 10));
    }

    /// Quilled Greatwurm counters up on combat damage during your turn.
    #[test]
    fn quilled_greatwurm_counters_on_combat_damage() {
        let mut g = two_player_game();
        g.active_player_idx = 0;
        let wurm = g.add_card_to_battlefield(0, catalog::quilled_greatwurm());
        g.fire_combat_damage_to_player_triggers(wurm, 1, 7);
        drain_stack(&mut g);
        assert_eq!(
            g.battlefield_find(wurm).unwrap().counter_count(CounterType::PlusOnePlusOne),
            7,
            "seven +1/+1 counters"
        );
    }
}

mod recent204 {
    use crabomination::card::{CardType, CreatureType, Keyword};
    use crabomination::catalog;
    use crabomination::game::types::Target;
    use crabomination::game::*;
    use crabomination::game::{drain_stack, two_player_game};
    use crabomination::mana::Color;

    /// Saw grants +2/+0 to the creature it equips.
    #[test]
    fn saw_grants_power_bonus() {
        let mut g = two_player_game();
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let saw = g.add_card_to_battlefield(0, catalog::saw());
        g.players[0].mana_pool.add_colorless(2);
        g.perform_action(GameAction::Equip { equipment: saw, target: bear }).expect("equip Saw");
        drain_stack(&mut g);
        let c = g.computed_permanent(bear).unwrap();
        assert_eq!((c.power, c.toughness), (4, 2), "+2/+0 on the 2/2");
    }

    /// Unable to Scream turns the enchanted creature into a 0/2 Toy artifact creature
    /// with no abilities.
    #[test]
    fn unable_to_scream_makes_a_toy() {
        let mut g = two_player_game();
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        let victim = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4 flyer
        let aura = g.add_card_to_hand(0, catalog::unable_to_scream());
        g.players[0].mana_pool.add(Color::Blue, 1);
        g.perform_action(GameAction::CastSpell {
            card_id: aura, target: Some(Target::Permanent(victim)),
            additional_targets: vec![], mode: None, x_value: None,
        })
        .expect("cast Unable to Scream");
        drain_stack(&mut g);
        let c = g.computed_permanent(victim).unwrap();
        assert_eq!((c.power, c.toughness), (0, 2), "base 0/2");
        assert!(!c.keywords.contains(&Keyword::Flying), "lost flying");
        assert!(c.card_types.contains(&CardType::Artifact), "now an artifact");
        assert!(c.subtypes.creature_types.contains(&CreatureType::Toy), "a Toy");
    }

    /// Sporogenic Infection edicts on enter and destroys the host when it's damaged.
    #[test]
    fn sporogenic_infection_edicts_then_destroys_on_damage() {
        let mut g = two_player_game();
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        // The opponent has the host plus a spare to lose to the edict.
        let host = g.add_card_to_battlefield(1, catalog::serra_angel());
        g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let aura = g.add_card_to_hand(0, catalog::sporogenic_infection());
        g.players[0].mana_pool.add(Color::Black, 1);
        g.players[0].mana_pool.add_colorless(1);
        g.perform_action(GameAction::CastSpell {
            card_id: aura, target: Some(Target::Permanent(host)),
            additional_targets: vec![], mode: None, x_value: None,
        })
        .expect("cast Sporogenic Infection");
        drain_stack(&mut g);
        assert_eq!(
            g.battlefield.iter().filter(|c| c.controller == 1 && c.definition.is_creature()).count(),
            1,
            "opponent sacrificed one creature to the edict"
        );
        // Damage the host → destroyed.
        let mut evs = Vec::new();
        g.deal_damage_to_from(crabomination::game::effects::EntityRef::Permanent(host), 1, None, &mut evs);
        g.dispatch_triggers_for_events(&evs);
        drain_stack(&mut g);
        let sbas = g.check_state_based_actions();
        g.dispatch_triggers_for_events(&sbas);
        drain_stack(&mut g);
        let _ = g.check_state_based_actions();
        assert!(g.battlefield_find(host).is_none(), "host destroyed after taking damage");
    }

    /// Under the Skin manifests dread and returns a permanent from the graveyard.
    #[test]
    fn under_the_skin_manifests_and_recurs() {
        let mut g = two_player_game();
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        for _ in 0..3 { g.add_card_to_library(0, catalog::grizzly_bears()); }
        g.add_card_to_graveyard(0, catalog::grizzly_bears());
        let spell = g.add_card_to_hand(0, catalog::under_the_skin());
        g.players[0].mana_pool.add(Color::Green, 1);
        g.players[0].mana_pool.add_colorless(2);
        let hand0 = g.players[0].hand.len();
        g.perform_action(GameAction::CastSpell {
            card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None,
        })
        .expect("cast Under the Skin");
        drain_stack(&mut g);
        // A face-down 2/2 manifested onto the battlefield.
        let facedown = g.battlefield.iter().filter(|c| c.controller == 0 && c.face_down).count();
        assert_eq!(facedown, 1, "one manifested face-down creature");
        // The graveyard permanent came back to hand (net: -1 spell +1 return = hand0).
        assert_eq!(g.players[0].hand.len(), hand0, "returned a permanent from the graveyard");
    }
}

mod recent205 {
    use crabomination::catalog;
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    use crabomination::game::types::Target;
    use crabomination::game::*;
    use crabomination::game::{drain_stack, two_player_game};
    use crabomination::mana::Color;

    /// Don't Make a Sound counters a spell whose controller can't pay {2}.
    #[test]
    fn dont_make_a_sound_counters_when_unpaid() {
        let mut g = two_player_game();
        let bear = g.add_card_to_hand(1, catalog::grizzly_bears());
        g.players[1].mana_pool.add(Color::Green, 1);
        g.players[1].mana_pool.add_colorless(1);
        g.active_player_idx = 1;
        g.priority.player_with_priority = 1;
        g.perform_action(GameAction::CastSpell {
            card_id: bear, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast bear");
        // P1 is now tapped out; P0 casts Don't Make a Sound at the bear.
        let counter = g.add_card_to_hand(0, catalog::dont_make_a_sound());
        g.players[0].mana_pool.add(Color::Blue, 1);
        g.players[0].mana_pool.add_colorless(1);
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::CastSpell {
            card_id: counter, target: Some(Target::Permanent(bear)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("soft-counter the bear");
        drain_stack(&mut g);
        assert!(g.battlefield_find(bear).is_none(), "bear countered (no {{2}} to pay)");
        assert!(g.players[1].graveyard.iter().any(|c| c.id == bear), "countered spell in graveyard");
    }

    /// Keys to the House tutors a basic land to hand for {1}, {T}, Sacrifice.
    #[test]
    fn keys_to_the_house_fetches_a_basic() {
        let mut g = two_player_game();
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        let forest = g.add_card_to_library(0, catalog::forest());
        let keys = g.add_card_to_battlefield(0, catalog::keys_to_the_house());
        g.players[0].mana_pool.add_colorless(1);
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(forest))]));
        g.perform_action(GameAction::ActivateAbility {
            card_id: keys, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
        }).expect("activate Keys");
        drain_stack(&mut g);
        assert!(g.players[0].hand.iter().any(|c| c.id == forest), "basic land fetched to hand");
        assert!(g.battlefield_find(keys).is_none(), "Keys sacrificed itself");
    }

    /// Osseous Sticktwister punishes each opponent for its power once delirium is on.
    #[test]
    fn osseous_sticktwister_delirium_punisher() {
        let mut g = two_player_game();
        let stick = g.add_card_to_battlefield(0, catalog::osseous_sticktwister());
        g.active_player_idx = 0;
        // Four card types in the graveyard → delirium.
        g.add_card_to_graveyard(0, catalog::grizzly_bears()); // creature
        g.add_card_to_graveyard(0, catalog::lightning_bolt()); // instant
        g.add_card_to_graveyard(0, catalog::island()); // land
        g.add_card_to_graveyard(0, catalog::rite_of_the_dragoncaller()); // enchantment
        g.players[1].hand.clear();
        let l1 = g.players[1].life;
        g.fire_step_triggers(TurnStep::End);
        drain_stack(&mut g);
        let _ = stick;
        assert_eq!(g.players[1].life, l1 - 2, "opponent took 2 (this creature's power)");
    }
}

mod recent206 {
    use crabomination::card::{CounterType, Keyword};
    use crabomination::catalog;
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    use crabomination::game::types::Target;
    use crabomination::game::*;
    use crabomination::game::{drain_stack, two_player_game};
    use crabomination::mana::Color;

    /// Swiftblade Vindicator is a french-vanilla double strike / vigilance / trample.
    #[test]
    fn swiftblade_vindicator_has_three_keywords() {
        let mut g = two_player_game();
        let v = g.add_card_to_battlefield(0, catalog::swiftblade_vindicator());
        let cp = g.computed_permanent(v).unwrap();
        for kw in [Keyword::DoubleStrike, Keyword::Vigilance, Keyword::Trample] {
            assert!(cp.keywords.contains(&kw), "has {kw:?}");
        }
    }

    /// Progenitus shuffles into its owner's library instead of dying.
    #[test]
    fn progenitus_shuffles_in_instead_of_dying() {
        let mut g = two_player_game();
        let p = g.add_card_to_battlefield(0, catalog::progenitus());
        assert!(g.computed_permanent(p).unwrap().keywords.contains(&Keyword::ProtectionFromEverything));
        // Drop ten -1/-1 counters so the 10/10 dies to SBA.
        g.battlefield_find_mut(p).unwrap().counters.insert(CounterType::MinusOneMinusOne, 10);
        g.check_state_based_actions();
        assert!(g.battlefield_find(p).is_none(), "the 0/0 died to SBA");
        assert!(g.players[0].library.iter().any(|c| c.id == p), "shuffled into library, not graveyard");
        assert!(!g.players[0].graveyard.iter().any(|c| c.id == p), "not in the graveyard");
    }

    /// Rune-Scarred Demon tutors any card to hand on ETB.
    #[test]
    fn rune_scarred_demon_tutors_on_etb() {
        let mut g = two_player_game();
        let target = g.add_card_to_library(0, catalog::lightning_bolt());
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(target))]));
        // Enter through the real ETB funnel so the self-source trigger fires.
        g.move_card_to_battlefield_for_test(0, catalog::rune_scarred_demon());
        drain_stack(&mut g);
        assert!(g.players[0].hand.iter().any(|c| c.id == target), "tutored the card to hand");
    }

    /// Micromancer only finds a cheap instant/sorcery.
    #[test]
    fn micromancer_finds_one_mv_instant() {
        let mut g = two_player_game();
        let bolt = g.add_card_to_library(0, catalog::lightning_bolt()); // MV 1 instant
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(bolt))]));
        g.move_card_to_battlefield_for_test(0, catalog::micromancer());
        drain_stack(&mut g);
        assert!(g.players[0].hand.iter().any(|c| c.id == bolt), "found the {{1}} instant");
    }

    /// Seismic Rupture hits grounded creatures but spares fliers.
    #[test]
    fn seismic_rupture_spares_fliers() {
        let mut g = two_player_game();
        let ground = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
        let flier = g.add_card_to_battlefield(1, catalog::mahamoti_djinn()); // 5/6 flying
        let spell = g.add_card_to_hand(0, catalog::seismic_rupture());
        g.players[0].mana_pool.add(Color::Red, 1);
        g.players[0].mana_pool.add_colorless(2);
        g.step = TurnStep::PreCombatMain;
        g.perform_action(GameAction::CastSpell {
            card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast Seismic Rupture");
        drain_stack(&mut g);
        assert!(g.battlefield_find(ground).is_none(), "grounded 2/2 took 2 and died");
        assert!(g.battlefield_find(flier).is_some(), "the flier was untouched");
    }

    /// An Offer You Can't Refuse counters a noncreature spell and gifts its
    /// controller two Treasures.
    #[test]
    fn an_offer_counters_and_gives_treasures() {
        let mut g = two_player_game();
        g.active_player_idx = 1;
        g.priority.player_with_priority = 1;
        let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
        g.players[1].mana_pool.add(Color::Red, 1);
        g.perform_action(GameAction::CastSpell {
            card_id: bolt, target: Some(Target::Player(0)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("opponent casts a bolt");
        let offer = g.add_card_to_hand(0, catalog::an_offer_you_cant_refuse());
        g.players[0].mana_pool.add(Color::Blue, 1);
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::CastSpell {
            card_id: offer, target: Some(Target::Permanent(bolt)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("counter the bolt");
        drain_stack(&mut g);
        assert_eq!(g.players[0].life, 20, "bolt countered — no damage");
        let treasures = g.battlefield.iter().filter(|c| c.controller == 1 && c.definition.name == "Treasure").count();
        assert_eq!(treasures, 2, "the bolt's controller got two Treasures");
    }

    /// Involuntary Employment steals a creature for the turn with haste + a Treasure.
    #[test]
    fn involuntary_employment_steals_with_haste() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        g.battlefield_find_mut(bear).unwrap().tapped = true;
        let spell = g.add_card_to_hand(0, catalog::involuntary_employment());
        g.players[0].mana_pool.add(Color::Red, 1);
        g.players[0].mana_pool.add_colorless(3);
        g.step = TurnStep::PreCombatMain;
        g.perform_action(GameAction::CastSpell {
            card_id: spell, target: Some(Target::Permanent(bear)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast Involuntary Employment");
        drain_stack(&mut g);
        let c = g.battlefield_find(bear).unwrap();
        assert_eq!(c.controller, 0, "gained control");
        assert!(!c.tapped, "untapped");
        assert!(g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Haste), "has haste");
        assert!(g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Treasure"), "made a Treasure");
    }

    /// Pilfer makes the opponent discard a nonland card of the caster's choosing.
    #[test]
    fn pilfer_discards_a_nonland() {
        let mut g = two_player_game();
        let spell_card = g.add_card_to_hand(1, catalog::lightning_bolt());
        g.add_card_to_hand(1, catalog::forest());
        let pilfer = g.add_card_to_hand(0, catalog::pilfer());
        g.players[0].mana_pool.add(Color::Black, 1);
        g.players[0].mana_pool.add_colorless(1);
        g.step = TurnStep::PreCombatMain;
        g.perform_action(GameAction::CastSpell {
            card_id: pilfer, target: Some(Target::Player(1)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast Pilfer");
        drain_stack(&mut g);
        assert!(g.players[1].graveyard.iter().any(|c| c.id == spell_card), "nonland was discarded");
    }

    /// Grow from the Ashes fetches one basic normally, two when kicked.
    #[test]
    fn grow_from_the_ashes_kicked_fetches_two() {
        let mut g = two_player_game();
        let f1 = g.add_card_to_library(0, catalog::forest());
        let f2 = g.add_card_to_library(0, catalog::forest());
        let spell = g.add_card_to_hand(0, catalog::grow_from_the_ashes());
        g.players[0].mana_pool.add(Color::Green, 1);
        g.players[0].mana_pool.add_colorless(4); // {2}{G} + kicker {2}
        g.step = TurnStep::PreCombatMain;
        g.decider = Box::new(ScriptedDecider::new([
            DecisionAnswer::Search(Some(f1)),
            DecisionAnswer::Search(Some(f2)),
        ]));
        g.perform_action(GameAction::CastSpellKicked {
            card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast kicked Grow from the Ashes");
        drain_stack(&mut g);
        assert!(g.battlefield_find(f1).is_some() && g.battlefield_find(f2).is_some(),
            "kicked fetch put both basics onto the battlefield");
    }

    /// Doubling Season doubles tokens an effect makes under your control.
    #[test]
    fn doubling_season_doubles_tokens() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::doubling_season());
        g.add_card_to_battlefield(0, catalog::rite_of_the_dragoncaller());
        let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
        g.players[0].mana_pool.add(Color::Red, 1);
        g.step = TurnStep::PreCombatMain;
        g.perform_action(GameAction::CastSpell {
            card_id: bolt, target: Some(Target::Player(1)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast a bolt to trigger Rite");
        drain_stack(&mut g);
        let dragons = g.battlefield.iter().filter(|c| c.definition.name == "Dragon").count();
        assert_eq!(dragons, 2, "Rite's one Dragon was doubled to two");
    }
}

mod recent207 {
    use crabomination::card::Keyword;
    use crabomination::catalog;
    use crabomination::game::types::Target;
    use crabomination::game::*;
    use crabomination::game::{drain_stack, two_player_game};
    use crabomination::mana::Color;

    /// Cast a no-target sorcery/instant from hand with the given mana, then drain.
    fn cast_simple(g: &mut GameState, card: CardId, colorless: u32) {
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g.players[0].mana_pool.add_colorless(colorless);
        g.perform_action(GameAction::CastSpell {
            card_id: card, target: None, additional_targets: vec![], mode: None, x_value: None,
        })
        .expect("cast");
        drain_stack(g);
    }

    /// Release the Dogs makes four 1/1 Dog tokens.
    #[test]
    fn release_the_dogs_makes_four() {
        let mut g = two_player_game();
        let s = g.add_card_to_hand(0, catalog::release_the_dogs());
        g.players[0].mana_pool.add(Color::White, 1);
        cast_simple(&mut g, s, 3);
        assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Dog").count(), 4);
    }

    /// Moment of Triumph pumps and gains life.
    #[test]
    fn moment_of_triumph_pumps_and_gains() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let s = g.add_card_to_hand(0, catalog::moment_of_triumph());
        g.players[0].mana_pool.add(Color::White, 1);
        g.step = TurnStep::PreCombatMain;
        let life = g.players[0].life;
        g.perform_action(GameAction::CastSpell {
            card_id: s, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast");
        drain_stack(&mut g);
        let cp = g.computed_permanent(bear).unwrap();
        assert_eq!((cp.power, cp.toughness), (4, 4));
        assert_eq!(g.players[0].life, life + 2);
    }

    /// Deadly Riposte only hits tapped creatures.
    #[test]
    fn deadly_riposte_burns_tapped() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        g.battlefield_find_mut(bear).unwrap().tapped = true;
        let s = g.add_card_to_hand(0, catalog::deadly_riposte());
        g.players[0].mana_pool.add(Color::White, 1);
        g.players[0].mana_pool.add_colorless(1);
        g.step = TurnStep::PreCombatMain;
        let life = g.players[0].life;
        g.perform_action(GameAction::CastSpell {
            card_id: s, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast");
        drain_stack(&mut g);
        assert!(g.battlefield_find(bear).is_none(), "tapped 2/2 took 3 and died");
        assert_eq!(g.players[0].life, life + 2);
    }

    /// Skeleton Archer pings any target on ETB.
    #[test]
    fn skeleton_archer_pings_on_etb() {
        let mut g = two_player_game();
        let l1 = g.players[1].life;
        // No creatures on the opponent's side, so the "any target" auto-picks the
        // opponent's face.
        g.move_card_to_battlefield_for_test(0, catalog::skeleton_archer());
        drain_stack(&mut g);
        assert_eq!(g.players[1].life, l1 - 1, "pinged the opponent for 1");
    }

    /// Maalfeld Twins leaves two Zombies when it dies.
    #[test]
    fn maalfeld_twins_dies_into_zombies() {
        let mut g = two_player_game();
        let twins = g.add_card_to_battlefield(0, catalog::maalfeld_twins());
        g.battlefield_find_mut(twins).unwrap().counters.insert(crabomination::card::CounterType::MinusOneMinusOne, 4);
        g.check_state_based_actions();
        drain_stack(&mut g);
        assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Zombie").count(), 2);
    }

    /// Rapacious Dragon mints two Treasures on ETB.
    #[test]
    fn rapacious_dragon_makes_treasures() {
        let mut g = two_player_game();
        g.move_card_to_battlefield_for_test(0, catalog::rapacious_dragon());
        drain_stack(&mut g);
        assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Treasure").count(), 2);
    }

    /// Exclusion Mage bounces an opponent's creature on ETB.
    #[test]
    fn exclusion_mage_bounces() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        g.move_card_to_battlefield_for_test(0, catalog::exclusion_mage());
        drain_stack(&mut g);
        assert!(g.battlefield_find(bear).is_none(), "bounced");
        assert!(g.players[1].hand.iter().any(|c| c.id == bear), "returned to owner's hand");
    }

    /// Mystic Archaeologist draws two for {3}{U}{U}.
    #[test]
    fn mystic_archaeologist_draws_two() {
        let mut g = two_player_game();
        let m = g.add_card_to_battlefield(0, catalog::mystic_archaeologist());
        g.add_card_to_library(0, catalog::island());
        g.add_card_to_library(0, catalog::island());
        g.players[0].mana_pool.add(Color::Blue, 2);
        g.players[0].mana_pool.add_colorless(3);
        let h = g.players[0].hand.len();
        g.perform_action(GameAction::ActivateAbility {
            card_id: m, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
        }).expect("activate");
        drain_stack(&mut g);
        assert_eq!(g.players[0].hand.len(), h + 2);
    }

    /// Deathmark destroys a green creature but can't target a blue one.
    #[test]
    fn deathmark_hits_green_not_blue() {
        let mut g = two_player_game();
        let birds = g.add_card_to_battlefield(1, catalog::birds_of_paradise()); // green
        let s = g.add_card_to_hand(0, catalog::deathmark());
        g.players[0].mana_pool.add(Color::Black, 1);
        g.step = TurnStep::PreCombatMain;
        g.perform_action(GameAction::CastSpell {
            card_id: s, target: Some(Target::Permanent(birds)), additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast");
        drain_stack(&mut g);
        assert!(g.battlefield_find(birds).is_none(), "green creature destroyed");
    }

    /// Goblin Oriflamme pumps only attacking creatures.
    #[test]
    fn goblin_oriflamme_pumps_attackers() {
        use crabomination::game::types::{Attack, AttackTarget};
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::goblin_oriflamme());
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.clear_sickness(bear);
        // Not attacking yet: no bonus.
        assert_eq!(g.computed_permanent(bear).unwrap().power, 2);
        g.step = TurnStep::DeclareAttackers;
        g.priority.player_with_priority = 0;
        g.declare_attackers(vec![Attack { attacker: bear, target: AttackTarget::Player(1) }])
            .expect("bear attacks");
        assert_eq!(g.computed_permanent(bear).unwrap().power, 3, "attacking gets +1/+0");
    }

    /// Vampire Neonate drains 1 for {2}, {T}.
    #[test]
    fn vampire_neonate_drains() {
        let mut g = two_player_game();
        let n = g.add_card_to_battlefield(0, catalog::vampire_neonate());
        g.clear_sickness(n);
        g.players[0].mana_pool.add_colorless(2);
        let (l0, l1) = (g.players[0].life, g.players[1].life);
        g.perform_action(GameAction::ActivateAbility {
            card_id: n, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
        }).expect("activate");
        drain_stack(&mut g);
        assert_eq!(g.players[1].life, l1 - 1);
        assert_eq!(g.players[0].life, l0 + 1);
    }

    /// Volley Veteran deals damage equal to your Goblin count.
    #[test]
    fn volley_veteran_scales_with_goblins() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::raging_redcap()); // a Goblin
        let target = g.add_card_to_battlefield(1, catalog::mahamoti_djinn()); // 5/6
        g.move_card_to_battlefield_for_test(0, catalog::volley_veteran()); // now 2 Goblins
        drain_stack(&mut g);
        // 2 Goblins → 2 damage to the 5/6.
        assert_eq!(g.battlefield_find(target).unwrap().damage, 2);
    }

    /// Regal Caracal buffs and lifelinks other Cats, and makes two Cat tokens.
    #[test]
    fn regal_caracal_lords_cats() {
        let mut g = two_player_game();
        g.move_card_to_battlefield_for_test(0, catalog::regal_caracal());
        drain_stack(&mut g);
        let cats: Vec<_> = g.battlefield.iter().filter(|c| c.definition.name == "Cat").map(|c| c.id).collect();
        assert_eq!(cats.len(), 2, "made two Cat tokens");
        // Each token is a base 1/1 lifelink; the lord makes them 2/2.
        let cp = g.computed_permanent(cats[0]).unwrap();
        assert_eq!((cp.power, cp.toughness), (2, 2), "lord buffs other Cats");
        assert!(cp.keywords.contains(&Keyword::Lifelink));
    }

    /// Harmless Offering donates a permanent to an opponent.
    #[test]
    fn harmless_offering_donates() {
        let mut g = two_player_game();
        let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let s = g.add_card_to_hand(0, catalog::harmless_offering());
        g.players[0].mana_pool.add(Color::Red, 1);
        g.players[0].mana_pool.add_colorless(2);
        g.step = TurnStep::PreCombatMain;
        g.perform_action(GameAction::CastSpell {
            card_id: s, target: Some(Target::Permanent(mine)), additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast");
        drain_stack(&mut g);
        assert_eq!(g.battlefield_find(mine).unwrap().controller, 1, "opponent now controls it");
    }

    /// Syr Alin buffs the rest of the team when it attacks.
    #[test]
    fn syr_alin_pumps_team_on_attack() {
        use crabomination::game::types::{Attack, AttackTarget};
        let mut g = two_player_game();
        let alin = g.add_card_to_battlefield(0, catalog::syr_alin_the_lions_claw());
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.clear_sickness(alin);
        g.step = TurnStep::DeclareAttackers;
        g.priority.player_with_priority = 0;
        let evs = g
            .declare_attackers(vec![Attack { attacker: alin, target: AttackTarget::Player(1) }])
            .expect("Syr Alin attacks");
        g.dispatch_triggers_for_events(&evs);
        drain_stack(&mut g);
        assert_eq!(g.computed_permanent(bear).unwrap().power, 3, "other creatures +1/+1");
        assert_eq!(g.computed_permanent(alin).unwrap().power, 4, "not itself");
    }

    /// Dive Down toughens a creature and grants hexproof.
    #[test]
    fn dive_down_grants_hexproof() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let s = g.add_card_to_hand(0, catalog::dive_down());
        g.players[0].mana_pool.add(Color::Blue, 1);
        g.step = TurnStep::PreCombatMain;
        g.perform_action(GameAction::CastSpell {
            card_id: s, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast");
        drain_stack(&mut g);
        let cp = g.computed_permanent(bear).unwrap();
        assert_eq!((cp.power, cp.toughness), (2, 5), "+0/+3");
        assert!(cp.keywords.contains(&Keyword::Hexproof));
    }

    /// Hidetsugu's Second Rite only burns a player who is at exactly 10 life.
    #[test]
    fn hidetsugus_second_rite_needs_exactly_ten() {
        let mut g = two_player_game();
        let s = g.add_card_to_hand(0, catalog::hidetsugus_second_rite());
        g.players[1].life = 11; // not exactly 10
        g.players[0].mana_pool.add(Color::Red, 1);
        g.players[0].mana_pool.add_colorless(3);
        g.step = TurnStep::PreCombatMain;
        g.perform_action(GameAction::CastSpell {
            card_id: s, target: Some(Target::Player(1)), additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast");
        drain_stack(&mut g);
        assert_eq!(g.players[1].life, 11, "not at 10 → no damage");

        let s2 = g.add_card_to_hand(0, catalog::hidetsugus_second_rite());
        g.players[1].life = 10;
        g.players[0].mana_pool.add(Color::Red, 1);
        g.players[0].mana_pool.add_colorless(3);
        g.perform_action(GameAction::CastSpell {
            card_id: s2, target: Some(Target::Player(1)), additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast");
        drain_stack(&mut g);
        assert_eq!(g.players[1].life, 0, "exactly 10 → dealt 10");
    }

    /// Rise of the Dark Realms reanimates every creature from all graveyards under
    /// your control (CR 608 mass move across all graveyards).
    #[test]
    fn rise_of_the_dark_realms_grabs_all_graveyards() {
        let mut g = two_player_game();
        let mine = g.add_card_to_graveyard(0, catalog::grizzly_bears());
        let theirs = g.add_card_to_graveyard(1, catalog::grizzly_bears());
        g.add_card_to_graveyard(1, catalog::lightning_bolt()); // not a creature — stays
        let s = g.add_card_to_hand(0, catalog::rise_of_the_dark_realms());
        g.players[0].mana_pool.add(Color::Black, 2);
        cast_simple(&mut g, s, 7);
        assert_eq!(g.battlefield_find(mine).map(|c| c.controller), Some(0), "my creature returned under my control");
        assert_eq!(g.battlefield_find(theirs).map(|c| c.controller), Some(0), "opponent's creature stolen under my control");
    }

    /// CR 702.16e — protection from everything prevents combat damage: Progenitus
    /// takes no damage from an attacker it blocks.
    #[test]
    fn progenitus_blocks_without_taking_damage() {
        use crabomination::game::types::{Attack, AttackTarget};
        let mut g = two_player_game();
        let attacker = g.add_card_to_battlefield(1, catalog::mahamoti_djinn()); // 5/6
        let prog = g.add_card_to_battlefield(0, catalog::progenitus()); // 10/10
        g.attacking = vec![Attack { attacker, target: AttackTarget::Player(0) }];
        g.block_map.insert(prog, attacker);
        g.step = TurnStep::CombatDamage;
        g.active_player_idx = 1;
        g.resolve_combat().expect("combat damage");
        drain_stack(&mut g);
        assert_eq!(g.battlefield_find(prog).map(|c| c.damage), Some(0), "protection from everything → no damage marked");
    }

    /// Magnigoth Sentry has reach; Raging Redcap has double strike (french vanillas).
    #[test]
    fn vanilla_keyword_bodies() {
        let mut g = two_player_game();
        let sentry = g.add_card_to_battlefield(0, catalog::magnigoth_sentry());
        let redcap = g.add_card_to_battlefield(0, catalog::raging_redcap());
        assert!(g.computed_permanent(sentry).unwrap().keywords.contains(&Keyword::Reach));
        assert!(g.computed_permanent(redcap).unwrap().keywords.contains(&Keyword::DoubleStrike));
    }
}
