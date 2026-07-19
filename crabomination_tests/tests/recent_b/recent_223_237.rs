//! Tests for recentN card batches 223-237 (merged from per-batch micro-files).

mod recent223 {
    use crabomination::card::Keyword;
    use crabomination::catalog;
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    use crabomination::game::effects::EffectContext;
    use crabomination::game::types::{Attack, AttackTarget, Target, TurnStep};
    use crabomination::game::{drain_stack, two_player_game, GameState};

    /// Warren Warleader carries Offspring and its attack trigger can mint a Rabbit.
    #[test]
    fn warren_warleader_attack_makes_a_rabbit() {
        let warleader = catalog::warren_warleader();
        assert!(warleader.keywords.iter().any(|k| matches!(k, Keyword::Offspring(_))), "has Offspring");
        let mut g = two_player_game();
        let wl = g.add_card_to_battlefield(0, catalog::warren_warleader());
        g.clear_sickness(wl);
        g.active_player_idx = 0;
        g.step = TurnStep::DeclareAttackers;
        g.priority.player_with_priority = 0;
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Mode(0)])); // make a Rabbit
        let rabbits = |g: &GameState| g.battlefield.iter().filter(|c| c.controller == 0 && c.definition.name == "Rabbit").count();
        g.declare_attackers(vec![Attack { attacker: wl, target: AttackTarget::Player(1) }]).expect("attack");
        drain_stack(&mut g);
        assert_eq!(rabbits(&g), 1, "attack mode 0 minted a Rabbit token");
    }

    /// For the Common Good makes X token copies, shields all your tokens, and gains
    /// life per token.
    #[test]
    fn for_the_common_good_copies_and_shields() {
        let mut g = two_player_game();
        let orig = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.battlefield_find_mut(orig).unwrap().is_token = true;
        let tokens = |g: &GameState| g.battlefield.iter().filter(|c| c.controller == 0 && c.is_token).count();
        let life = g.players[0].life;
        let effect = catalog::for_the_common_good().effect.clone();
        let ctx = EffectContext {
            x_value: 2,
            targets: vec![Target::Permanent(orig)],
            ..EffectContext::for_ability(orig, 0, None)
        };
        g.resolve_effect(&effect, &ctx).unwrap();
        drain_stack(&mut g);
        assert_eq!(tokens(&g), 3, "two copies join the original token");
        assert_eq!(g.players[0].life, life + 3, "gained 1 life per token");
        assert!(g.computed_permanent(orig).unwrap().keywords.contains(&Keyword::Indestructible), "tokens shielded");
    }
}

mod recent224 {
    use crabomination::card::Keyword;
    use crabomination::catalog;
    use crabomination::game::effects::EffectContext;
    use crabomination::game::types::Target;
    use crabomination::game::two_player_game;

    /// Long River Lurker's ETB makes one of your creatures unblockable.
    #[test]
    fn long_river_lurker_makes_unblockable() {
        let mut g = two_player_game();
        let ally = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let lurker = g.add_card_to_battlefield(0, catalog::long_river_lurker());
        let effect = catalog::long_river_lurker().triggered_abilities[0].effect.clone();
        let ctx = EffectContext { targets: vec![Target::Permanent(ally)], ..EffectContext::for_trigger(lurker, 0, None, 0) };
        g.resolve_effect(&effect, &ctx).unwrap();
        assert!(g.computed_permanent(ally).unwrap().keywords.contains(&Keyword::Unblockable), "ally can't be blocked");
        // The Lurker itself has Ward.
        assert!(g.computed_permanent(lurker).unwrap().keywords.iter().any(|k| matches!(k, Keyword::Ward(_))), "Lurker has ward");
    }

    /// Kolodin grants haste to Vehicles you control.
    #[test]
    fn kolodin_gives_vehicles_haste() {
        let mut g = two_player_game();
        let vehicle = g.add_card_to_battlefield(0, catalog::tangle_tumbler());
        assert!(!g.computed_permanent(vehicle).unwrap().keywords.contains(&Keyword::Haste), "no haste on its own");
        g.add_card_to_battlefield(0, catalog::kolodin_triumph_caster());
        assert!(g.computed_permanent(vehicle).unwrap().keywords.contains(&Keyword::Haste), "Kolodin grants haste");
    }

    /// Mu Yanling makes a Vehicle on entry and that Vehicle flies under her static.
    #[test]
    fn mu_yanling_makes_a_flying_vehicle() {
        let mut g = two_player_game();
        let mu = g.add_card_to_battlefield(0, catalog::mu_yanling_wind_rider());
        let etb = catalog::mu_yanling_wind_rider().triggered_abilities[0].effect.clone();
        g.resolve_effect(&etb, &EffectContext::for_trigger(mu, 0, None, 0)).unwrap();
        let vehicle = g.battlefield.iter().find(|c| c.controller == 0 && c.definition.name == "Vehicle").map(|c| c.id).expect("a Vehicle token entered");
        assert!(g.computed_permanent(vehicle).unwrap().keywords.contains(&Keyword::Flying), "the Vehicle flies under Mu Yanling");
    }
}

mod recent225 {
    use crabomination::card::ArtifactSubtype;
    use crabomination::catalog;
    use crabomination::game::effects::EffectContext;
    use crabomination::game::types::Target;
    use crabomination::game::{drain_stack, two_player_game, GameAction};
    use crabomination::mana::Color;

    /// Savage Ventmaw's attack mana survives step/phase emptying this turn but is
    /// gone at cleanup (CR 500.4 exception).
    #[test]
    fn savage_ventmaw_mana_persists_this_turn() {
        let mut g = two_player_game();
        let vent = g.add_card_to_battlefield(0, catalog::savage_ventmaw());
        let effect = catalog::savage_ventmaw().triggered_abilities[0].effect.clone();
        g.resolve_effect(&effect, &EffectContext::for_trigger(vent, 0, None, 0)).unwrap();
        assert_eq!(g.players[0].mana_pool.amount(Color::Red), 3);
        assert_eq!(g.players[0].mana_pool.amount(Color::Green), 3);
        // A step/phase boundary empties pools — the kept mana re-seeds.
        g.empty_mana_pools();
        assert_eq!(g.players[0].mana_pool.total(), 6, "kept mana survives the empty");
        // Cleanup clears the kept record, so the turn's final empty removes it.
        g.players[0].kept_mana_this_turn.empty();
        g.empty_mana_pools();
        assert_eq!(g.players[0].mana_pool.total(), 0, "kept mana gone at end of turn");
    }

    /// Fake Your Own Death: the pumped creature returns tapped and mints a Treasure
    /// when it dies this turn.
    #[test]
    fn fake_your_own_death_revives_with_treasure() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let spell = g.add_card_to_hand(0, catalog::fake_your_own_death());
        g.players[0].mana_pool.add(Color::Black, 1);
        g.players[0].mana_pool.add_colorless(1);
        g.perform_action(GameAction::CastSpell {
            card_id: spell, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None,
        })
        .expect("castable for {1}{B}");
        drain_stack(&mut g);
        assert_eq!(g.computed_permanent(bear).unwrap().power, 4, "+2/+0");
        // Kill the bear; its granted trigger should return it tapped + make a Treasure.
        let ctx = EffectContext { targets: vec![Target::Permanent(bear)], ..EffectContext::for_ability(bear, 0, None) };
        g.resolve_effect(&crabomination::effect::Effect::Destroy { what: crabomination::effect::Selector::Target(0) }, &ctx).unwrap();
        drain_stack(&mut g);
        let revived = g.battlefield.iter().find(|c| c.id == bear).expect("bear back on battlefield");
        assert!(revived.tapped, "returned tapped");
        assert!(
            g.battlefield.iter().any(|c| c.controller == 0
                && c.definition.subtypes.artifact_subtypes.contains(&ArtifactSubtype::Treasure)),
            "a Treasure was created",
        );
    }

    fn count_named(g: &crabomination::game::GameState, ctrl: usize, name: &str) -> usize {
        g.battlefield.iter().filter(|c| c.controller == ctrl && c.definition.name == name).count()
    }

    /// Dread Summons makes a Zombie for each creature card milled.
    #[test]
    fn dread_summons_mills_and_makes_zombies() {
        let mut g = two_player_game();
        g.add_card_to_library(0, catalog::grizzly_bears());
        g.add_card_to_library(0, catalog::grizzly_bears());
        let effect = catalog::dread_summons().effect.clone();
        g.resolve_effect(&effect, &EffectContext::for_spell(0, None, 0, 2)).unwrap();
        assert_eq!(count_named(&g, 0, "Zombie"), 2, "one Zombie per creature card milled");
    }

    /// On the Job pumps your team and investigates.
    #[test]
    fn on_the_job_pumps_and_investigates() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.resolve_effect(&catalog::on_the_job().effect.clone(), &EffectContext::for_spell(0, None, 0, 0)).unwrap();
        assert_eq!(g.computed_permanent(bear).unwrap().power, 4, "+2/+1");
        assert_eq!(count_named(&g, 0, "Clue"), 1, "investigated");
    }

    /// Makeshift Binding exiles an opponent's creature and gains 2 life.
    #[test]
    fn makeshift_binding_exiles_and_gains() {
        let mut g = two_player_game();
        let enemy = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let src = g.add_card_to_battlefield(0, catalog::makeshift_binding());
        let effect = catalog::makeshift_binding().triggered_abilities[0].effect.clone();
        let ctx = EffectContext { targets: vec![Target::Permanent(enemy)], ..EffectContext::for_trigger(src, 0, None, 0) };
        g.resolve_effect(&effect, &ctx).unwrap();
        assert!(g.exile.iter().any(|c| c.id == enemy), "enemy exiled");
        assert_eq!(g.players[0].life, 22, "gained 2 life");
    }

    /// Long Goodbye destroys a cheap creature.
    #[test]
    fn long_goodbye_destroys_cheap_creature() {
        let mut g = two_player_game();
        let enemy = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let ctx = EffectContext { targets: vec![Target::Permanent(enemy)], ..EffectContext::for_spell(0, None, 0, 0) };
        g.resolve_effect(&catalog::long_goodbye().effect.clone(), &ctx).unwrap();
        drain_stack(&mut g);
        assert!(!g.battlefield.iter().any(|c| c.id == enemy), "destroyed");
        assert!(catalog::long_goodbye().keywords.contains(&crabomination::card::Keyword::CantBeCountered));
    }

    /// It Doesn't Add Up reanimates a creature and suspects it.
    #[test]
    fn it_doesnt_add_up_reanimates_and_suspects() {
        let mut g = two_player_game();
        let dead = g.add_card_to_graveyard(0, catalog::grizzly_bears());
        let ctx = EffectContext { targets: vec![Target::Permanent(dead)], ..EffectContext::for_spell(0, None, 0, 0) };
        g.resolve_effect(&catalog::it_doesnt_add_up().effect.clone(), &ctx).unwrap();
        let back = g.battlefield.iter().find(|c| c.id == dead).expect("reanimated");
        assert!(back.suspected, "suspected");
    }

    /// Eliminate the Impossible shrinks the opponent's board and investigates.
    #[test]
    fn eliminate_the_impossible_shrinks_opponents() {
        let mut g = two_player_game();
        let enemy = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        g.resolve_effect(&catalog::eliminate_the_impossible().effect.clone(), &EffectContext::for_spell(0, None, 0, 0)).unwrap();
        assert_eq!(g.computed_permanent(enemy).unwrap().power, 0, "-2/-0");
        assert_eq!(count_named(&g, 0, "Clue"), 1, "investigated");
    }

    /// Unauthorized Exit bounces a nonland permanent.
    #[test]
    fn unauthorized_exit_bounces() {
        let mut g = two_player_game();
        let enemy = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let ctx = EffectContext { targets: vec![Target::Permanent(enemy)], ..EffectContext::for_spell(0, None, 0, 0) };
        g.resolve_effect(&catalog::unauthorized_exit().effect.clone(), &ctx).unwrap();
        assert!(g.players[1].hand.iter().any(|c| c.id == enemy), "returned to owner's hand");
    }

    /// Seraphic Steed's saddled attack mints a flying Angel.
    #[test]
    fn seraphic_steed_makes_angel() {
        let mut g = two_player_game();
        let steed = g.add_card_to_battlefield(0, catalog::seraphic_steed());
        let effect = catalog::seraphic_steed().triggered_abilities[0].effect.clone();
        g.resolve_effect(&effect, &EffectContext::for_trigger(steed, 0, None, 0)).unwrap();
        let angel = g.battlefield.iter().find(|c| c.definition.name == "Angel").expect("angel token");
        assert!(g.computed_permanent(angel.id).unwrap().keywords.contains(&crabomination::card::Keyword::Flying));
    }

    /// Sandstorm Salvager's ETB makes a 3/3 Golem.
    #[test]
    fn sandstorm_salvager_makes_golem() {
        let mut g = two_player_game();
        let sal = g.add_card_to_battlefield(0, catalog::sandstorm_salvager());
        let effect = catalog::sandstorm_salvager().triggered_abilities[0].effect.clone();
        g.resolve_effect(&effect, &EffectContext::for_trigger(sal, 0, None, 0)).unwrap();
        assert_eq!(count_named(&g, 0, "Golem"), 1, "a Golem entered");
    }

    /// Nightdrinker Moroii costs 3 life on entry.
    #[test]
    fn nightdrinker_moroii_loses_life() {
        let mut g = two_player_game();
        let m = g.add_card_to_battlefield(0, catalog::nightdrinker_moroii());
        let effect = catalog::nightdrinker_moroii().triggered_abilities[0].effect.clone();
        g.resolve_effect(&effect, &EffectContext::for_trigger(m, 0, None, 0)).unwrap();
        assert_eq!(g.players[0].life, 17, "lost 3 life");
    }

    /// Mirage Mesa taps for its chosen color.
    #[test]
    fn mirage_mesa_taps_for_chosen_color() {
        let mut g = two_player_game();
        let land = g.add_card_to_battlefield(0, catalog::mirage_mesa());
        g.resolve_effect(&crabomination::effect::Effect::ChooseColorForSelf, &EffectContext::for_ability(land, 0, None)).unwrap();
        let mana = catalog::mirage_mesa().activated_abilities[0].effect.clone();
        g.resolve_effect(&mana, &EffectContext::for_ability(land, 0, None)).unwrap();
        assert_eq!(g.players[0].mana_pool.total(), 1, "produced one mana of the chosen color");
    }

    /// Valgavoth's Lair is hexproof.
    #[test]
    fn valgavoths_lair_is_hexproof() {
        let mut g = two_player_game();
        let land = g.add_card_to_battlefield(0, catalog::valgavoths_lair());
        assert!(g.computed_permanent(land).unwrap().keywords.contains(&crabomination::card::Keyword::Hexproof));
    }

    /// Sandstorm Verge can stop a blocker.
    #[test]
    fn sandstorm_verge_prevents_block() {
        let mut g = two_player_game();
        let enemy = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let land = g.add_card_to_battlefield(0, catalog::sandstorm_verge());
        let effect = catalog::sandstorm_verge().activated_abilities[1].effect.clone();
        let ctx = EffectContext { targets: vec![Target::Permanent(enemy)], ..EffectContext::for_ability(land, 0, None) };
        g.resolve_effect(&effect, &ctx).unwrap();
        assert!(g.computed_permanent(enemy).unwrap().keywords.contains(&crabomination::card::Keyword::CantBlock));
    }

    /// Pitiless Carnage is plottable.
    #[test]
    fn pitiless_carnage_is_plottable() {
        assert!(catalog::pitiless_carnage().plot_cost.is_some());
    }

    /// Seasoned Consultant's battalion trigger pumps it.
    #[test]
    fn seasoned_consultant_battalion_pumps() {
        let mut g = two_player_game();
        let c = g.add_card_to_battlefield(0, catalog::seasoned_consultant());
        let effect = catalog::seasoned_consultant().triggered_abilities[0].effect.clone();
        g.resolve_effect(&effect, &EffectContext::for_trigger(c, 0, None, 0)).unwrap();
        assert_eq!(g.computed_permanent(c).unwrap().power, 3, "1 + 2 = 3");
    }

    /// Terramorphic Expanse fetches a basic land tapped.
    #[test]
    fn terramorphic_expanse_fetches_basic() {
        let mut g = two_player_game();
        let forest = g.add_card_to_library(0, catalog::forest());
        let land = g.add_card_to_battlefield(0, catalog::terramorphic_expanse());
        g.decider = Box::new(crabomination::decision::ScriptedDecider::new([
            crabomination::decision::DecisionAnswer::Search(Some(forest)),
        ]));
        let effect = catalog::terramorphic_expanse().activated_abilities[0].effect.clone();
        g.resolve_effect(&effect, &EffectContext::for_ability(land, 0, None)).unwrap();
        assert!(g.battlefield.iter().any(|c| c.definition.name == "Forest" && c.tapped), "fetched Forest tapped");
    }

    /// Wojek Investigator investigates on the trigger.
    #[test]
    fn wojek_investigator_investigates() {
        let mut g = two_player_game();
        let w = g.add_card_to_battlefield(0, catalog::wojek_investigator());
        let effect = catalog::wojek_investigator().triggered_abilities[0].effect.clone();
        g.resolve_effect(&effect, &EffectContext::for_trigger(w, 0, None, 0)).unwrap();
        assert_eq!(count_named(&g, 0, "Clue"), 1, "investigated");
    }
}

mod recent226 {
    use crabomination::card::Keyword;
    use crabomination::catalog;
    use crabomination::game::effects::EffectContext;
    use crabomination::game::types::Target;
    use crabomination::game::two_player_game;

    /// Doorkeeper Thrull suppresses an entering artifact's ETB trigger.
    #[test]
    fn doorkeeper_thrull_suppresses_artifact_etb() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::doorkeeper_thrull());
        // Sandstorm Salvager's ETB (make a Golem) must not fire under the suppressor.
        let sal = g.add_card_to_battlefield(0, catalog::sandstorm_salvager());
        let etb_fired = crabomination::game::actions::etb_trigger_multiplier(&g, 0, Some(sal));
        assert_eq!(etb_fired, 0, "artifact/creature ETB triggers suppressed");
    }

    /// Sanctuary Wall taps and stuns a target and itself.
    #[test]
    fn sanctuary_wall_taps_and_stuns() {
        let mut g = two_player_game();
        let enemy = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let wall = g.add_card_to_battlefield(0, catalog::sanctuary_wall());
        let effect = catalog::sanctuary_wall().activated_abilities[0].effect.clone();
        let ctx = EffectContext { targets: vec![Target::Permanent(enemy)], ..EffectContext::for_ability(wall, 0, None) };
        g.resolve_effect(&effect, &ctx).unwrap();
        assert!(g.battlefield_find(enemy).unwrap().tapped, "target tapped");
        assert!(g.battlefield_find(enemy).unwrap().counter_count(crabomination::card::CounterType::Stun) > 0);
        assert!(g.battlefield_find(wall).unwrap().counter_count(crabomination::card::CounterType::Stun) > 0);
    }

    /// All-Out Assault buffs the team with +1/+1 and deathtouch.
    #[test]
    fn all_out_assault_anthem_and_deathtouch() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.add_card_to_battlefield(0, catalog::all_out_assault());
        let cp = g.computed_permanent(bear).unwrap();
        assert_eq!(cp.power, 3, "+1/+1");
        assert!(cp.keywords.contains(&Keyword::Deathtouch));
    }

    /// Homicide Investigator investigates when a creature you control dies.
    #[test]
    fn homicide_investigator_investigates_on_death() {
        let mut g = two_player_game();
        let inv = g.add_card_to_battlefield(0, catalog::homicide_investigator());
        let effect = catalog::homicide_investigator().triggered_abilities[0].effect.clone();
        g.resolve_effect(&effect, &EffectContext::for_trigger(inv, 0, None, 0)).unwrap();
        assert_eq!(
            g.battlefield.iter().filter(|c| c.controller == 0 && c.definition.name == "Clue").count(),
            1,
        );
    }

    /// Lead Pipe grants +2/+0 to the creature it equips.
    #[test]
    fn lead_pipe_buffs_equipped() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let pipe = g.add_card_to_battlefield(0, catalog::lead_pipe());
        g.battlefield_find_mut(pipe).unwrap().attached_to = Some(bear);
        assert_eq!(g.computed_permanent(bear).unwrap().power, 4, "2 + 2 = 4");
    }

    /// Karlov Watchdog's battalion pumps the team.
    #[test]
    fn karlov_watchdog_battalion_pumps() {
        let mut g = two_player_game();
        let dog = g.add_card_to_battlefield(0, catalog::karlov_watchdog());
        let effect = catalog::karlov_watchdog().triggered_abilities[0].effect.clone();
        g.resolve_effect(&effect, &EffectContext::for_trigger(dog, 0, None, 0)).unwrap();
        assert_eq!(g.computed_permanent(dog).unwrap().power, 4, "3 + 1 = 4");
    }

    /// No Witnesses wraths the board and investigates.
    #[test]
    fn no_witnesses_destroys_all_creatures() {
        let mut g = two_player_game();
        let a = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let b = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        g.resolve_effect(&catalog::no_witnesses().effect.clone(), &EffectContext::for_spell(0, None, 0, 0)).unwrap();
        crabomination::game::drain_stack(&mut g);
        assert!(!g.battlefield.iter().any(|c| c.id == a || c.id == b), "all creatures destroyed");
        assert_eq!(g.battlefield.iter().filter(|c| c.controller == 0 && c.definition.name == "Clue").count(), 1);
    }
}

mod recent227 {
    use crabomination::catalog;
    use crabomination::game::effects::EffectContext;
    use crabomination::game::{drain_stack, two_player_game, GameAction};
    use crabomination::mana::Color;

    /// Persuasive Interrogators poisons an opponent when you sacrifice a Clue.
    #[test]
    fn persuasive_interrogators_poisons_on_clue_sac() {
        let mut g = two_player_game();
        let pi = g.add_card_to_battlefield(0, catalog::persuasive_interrogators());
        // The Clue-sac trigger (index 1) adds two poison to the opponent.
        let effect = catalog::persuasive_interrogators().triggered_abilities[1].effect.clone();
        g.resolve_effect(&effect, &EffectContext::for_trigger(pi, 0, None, 1)).unwrap();
        assert_eq!(g.players[1].poison_counters, 2, "opponent got two poison");
    }

    /// Perimeter Enforcer grows when another Detective enters.
    #[test]
    fn perimeter_enforcer_grows_on_detective_enter() {
        let mut g = two_player_game();
        let pe = g.add_card_to_battlefield(0, catalog::perimeter_enforcer());
        let effect = catalog::perimeter_enforcer().triggered_abilities[0].effect.clone();
        g.resolve_effect(&effect, &EffectContext::for_trigger(pe, 0, None, 0)).unwrap();
        assert_eq!(g.computed_permanent(pe).unwrap().power, 2, "1 + 1 = 2");
    }

    /// Visage Bandit enters as a copy of a creature you control.
    #[test]
    fn visage_bandit_enters_as_copy() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::savage_ventmaw()); // a 4/4 flier to copy
        let id = g.add_card_to_hand(0, catalog::visage_bandit());
        g.players[0].mana_pool.add(Color::Blue, 1);
        g.players[0].mana_pool.add_colorless(3);
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
        })
        .expect("cast Visage Bandit for {3}{U}");
        drain_stack(&mut g);
        let cp = g.computed_permanent(id).expect("copy survived");
        assert_eq!(cp.power, 4, "copied the 4/4");
        assert!(cp.keywords.contains(&crabomination::card::Keyword::Flying), "copied Flying");
    }
}

mod recent228 {
    use crabomination::card::Keyword;
    use crabomination::catalog;
    use crabomination::game::effects::EffectContext;
    use crabomination::game::types::Target;
    use crabomination::game::{drain_stack, two_player_game, GameAction};
    use crabomination::mana::Color;

    fn count_named(g: &crabomination::game::GameState, ctrl: usize, name: &str) -> usize {
        g.battlefield.iter().filter(|c| c.controller == ctrl && c.definition.name == name).count()
    }

    /// Pearl Medallion makes a white spell cost {1} less — a {1}{W} creature is
    /// castable off a single {W}.
    #[test]
    fn pearl_medallion_reduces_white_spells() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::pearl_medallion());
        let spell = g.add_card_to_hand(0, catalog::seasoned_consultant()); // {1}{W}
        g.players[0].mana_pool.add(Color::White, 1);
        g.perform_action(GameAction::CastSpell {
            card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None,
        })
        .expect("castable for {W} after the {1} reduction");
    }

    /// All five Medallions carry a matching-color cost reduction.
    #[test]
    fn medallions_all_reduce_their_color() {
        use crabomination::effect::StaticEffect;
        use crabomination::card::SelectionRequirement as R;
        for (f, c) in [
            (catalog::pearl_medallion as fn() -> _, Color::White),
            (catalog::sapphire_medallion, Color::Blue),
            (catalog::jet_medallion, Color::Black),
            (catalog::ruby_medallion, Color::Red),
            (catalog::emerald_medallion, Color::Green),
        ] {
            let def = f();
            assert!(def.static_abilities.iter().any(|s| matches!(
                &s.effect,
                StaticEffect::CostReduction { filter: R::HasColor(col), amount: 1 } if *col == c
            )));
        }
    }

    /// Meteoric Mace grants +4/+0 and trample to the creature it equips.
    #[test]
    fn meteoric_mace_buffs_equipped() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let mace = g.add_card_to_battlefield(0, catalog::meteoric_mace());
        g.battlefield_find_mut(mace).unwrap().attached_to = Some(bear);
        let cp = g.computed_permanent(bear).unwrap();
        assert_eq!(cp.power, 6, "2 + 4 = 6");
        assert!(cp.keywords.contains(&Keyword::Trample));
    }

    /// Reef Worm leaves a 3/3 Fish when it dies.
    #[test]
    fn reef_worm_makes_a_fish() {
        let mut g = two_player_game();
        let worm = g.add_card_to_battlefield(0, catalog::reef_worm());
        let effect = catalog::reef_worm().triggered_abilities[0].effect.clone();
        g.resolve_effect(&effect, &EffectContext::for_trigger(worm, 0, None, 0)).unwrap();
        assert_eq!(count_named(&g, 0, "Fish"), 1, "a Fish swims up");
    }

    /// Deserted Temple untaps a tapped land.
    #[test]
    fn deserted_temple_untaps_a_land() {
        let mut g = two_player_game();
        let forest = g.add_card_to_battlefield(0, catalog::forest());
        g.battlefield_find_mut(forest).unwrap().tapped = true;
        let temple = g.add_card_to_battlefield(0, catalog::deserted_temple());
        let effect = catalog::deserted_temple().activated_abilities[1].effect.clone();
        let ctx = EffectContext { targets: vec![Target::Permanent(forest)], ..EffectContext::for_ability(temple, 0, None) };
        g.resolve_effect(&effect, &ctx).unwrap();
        assert!(!g.battlefield_find(forest).unwrap().tapped, "land untapped");
    }

    /// Barbarian Ring pings you for its red mana.
    #[test]
    fn barbarian_ring_pings_you() {
        let mut g = two_player_game();
        let ring = g.add_card_to_battlefield(0, catalog::barbarian_ring());
        let effect = catalog::barbarian_ring().activated_abilities[0].effect.clone();
        g.resolve_effect(&effect, &EffectContext::for_ability(ring, 0, None)).unwrap();
        assert_eq!(g.players[0].mana_pool.amount(Color::Red), 1);
        assert_eq!(g.players[0].life, 19, "1 damage to you");
    }

    /// Glass Casket exiles a cheap opposing creature.
    #[test]
    fn glass_casket_exiles_cheap_creature() {
        let mut g = two_player_game();
        let enemy = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // mv 2
        let casket = g.add_card_to_battlefield(0, catalog::glass_casket());
        let effect = catalog::glass_casket().triggered_abilities[0].effect.clone();
        let ctx = EffectContext { targets: vec![Target::Permanent(enemy)], ..EffectContext::for_trigger(casket, 0, None, 0) };
        g.resolve_effect(&effect, &ctx).unwrap();
        assert!(g.exile.iter().any(|c| c.id == enemy), "creature exiled");
    }

    /// Crystal Grotto's second ability makes one mana of any color.
    #[test]
    fn crystal_grotto_makes_any_color() {
        let mut g = two_player_game();
        let grotto = g.add_card_to_battlefield(0, catalog::crystal_grotto());
        let effect = catalog::crystal_grotto().activated_abilities[1].effect.clone();
        g.resolve_effect(&effect, &EffectContext::for_ability(grotto, 0, None)).unwrap();
        assert_eq!(g.players[0].mana_pool.total(), 1, "one mana produced");
    }

    /// Molten Duplication mints a hasty artifact copy.
    #[test]
    fn molten_duplication_copies_with_haste() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let ctx = EffectContext { targets: vec![Target::Permanent(bear)], ..EffectContext::for_spell(0, None, 0, 0) };
        g.resolve_effect(&catalog::molten_duplication().effect.clone(), &ctx).unwrap();
        let copy = g.battlefield.iter().find(|c| c.is_token && c.definition.name == "Grizzly Bears").expect("token copy");
        let cp = g.computed_permanent(copy.id).unwrap();
        assert!(cp.keywords.contains(&Keyword::Haste), "gains haste");
        assert!(cp.card_types.contains(&crabomination::card::CardType::Artifact), "also an artifact");
    }

    /// Shackle Slinger taps an untapped opposing creature.
    #[test]
    fn shackle_slinger_taps_untapped() {
        let mut g = two_player_game();
        let enemy = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let ss = g.add_card_to_battlefield(0, catalog::shackle_slinger());
        let effect = catalog::shackle_slinger().triggered_abilities[0].effect.clone();
        let ctx = EffectContext { targets: vec![Target::Permanent(enemy)], ..EffectContext::for_trigger(ss, 0, None, 0) };
        g.resolve_effect(&effect, &ctx).unwrap();
        assert!(g.battlefield_find(enemy).unwrap().tapped, "untapped creature gets tapped");
    }

    /// Thunder Salvo deals at least 2 (base) to a creature.
    #[test]
    fn thunder_salvo_burns_a_creature() {
        let mut g = two_player_game();
        let enemy = g.add_card_to_battlefield(1, catalog::reef_worm()); // 0/1
        let ctx = EffectContext { targets: vec![Target::Permanent(enemy)], ..EffectContext::for_spell(0, None, 0, 0) };
        g.resolve_effect(&catalog::thunder_salvo().effect.clone(), &ctx).unwrap();
        drain_stack(&mut g);
        assert!(!g.battlefield.iter().any(|c| c.id == enemy), "0/1 dies to the salvo");
    }

    /// Fledgling Dragon grows to 5/5 with threshold active.
    #[test]
    fn fledgling_dragon_grows_with_threshold() {
        let mut g = two_player_game();
        let drag = g.add_card_to_battlefield(0, catalog::fledgling_dragon());
        assert_eq!(g.computed_permanent(drag).unwrap().power, 2, "2/2 without threshold");
        for _ in 0..7 {
            g.add_card_to_graveyard(0, catalog::grizzly_bears());
        }
        assert_eq!(g.computed_permanent(drag).unwrap().power, 5, "+3/+3 with seven cards in gy");
    }

    /// Annoyed Altisaur has reach, trample, and a cascade trigger.
    #[test]
    fn annoyed_altisaur_has_cascade() {
        let def = catalog::annoyed_altisaur();
        assert!(def.keywords.contains(&Keyword::Reach) && def.keywords.contains(&Keyword::Trample));
        assert!(!def.triggered_abilities.is_empty(), "carries a cascade trigger");
    }
}

mod recent229 {
    use crabomination::card::{CounterType, Keyword};
    use crabomination::catalog;
    use crabomination::game::effects::EffectContext;
    use crabomination::game::types::Target;
    use crabomination::game::{drain_stack, two_player_game, GameAction, TurnStep};

    /// Primal Might pumps your creature by X and fights the enemy target: a 2/2
    /// pumped +3/+3 (5/5) kills an opposing 4/4 and survives.
    #[test]
    fn primal_might_pumps_then_fights() {
        let mut g = two_player_game();
        let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
        let theirs = g.add_card_to_battlefield(1, catalog::hill_giant()); // 3/3
        let ctx = EffectContext {
            targets: vec![Target::Permanent(mine), Target::Permanent(theirs)],
            ..EffectContext::for_spell(0, None, 0, 3)
        };
        g.resolve_effect(&catalog::primal_might().effect.clone(), &ctx).unwrap();
        drain_stack(&mut g);
        assert!(!g.battlefield.iter().any(|c| c.id == theirs), "5/5 kills the 3/3");
        let cp = g.computed_permanent(mine).unwrap();
        assert_eq!(cp.power, 5, "2 + 3 = 5");
        assert!(g.battlefield.iter().any(|c| c.id == mine), "survives 3 damage on 5 toughness");
    }

    /// Boom Box destroys up to one artifact, creature, and land — all three when
    /// supplied. Every slot is optional (min 0).
    #[test]
    fn boom_box_destroys_each_type() {
        let def = catalog::boom_box();
        let ability = def.activated_abilities[0].effect.clone();
        assert!(ability.target_slot_optional(0, None), "up to one — every slot declinable");

        let mut g = two_player_game();
        let art = g.add_card_to_battlefield(1, catalog::mind_stone());
        let cre = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let land = g.add_card_to_battlefield(1, catalog::forest());
        let ctx = EffectContext {
            targets: vec![Target::Permanent(art), Target::Permanent(cre), Target::Permanent(land)],
            ..EffectContext::for_ability(art, 0, None)
        };
        g.resolve_effect(&ability, &ctx).unwrap();
        drain_stack(&mut g);
        for id in [art, cre, land] {
            assert!(!g.battlefield.iter().any(|c| c.id == id), "target destroyed");
        }
    }

    /// Prizefight fights the two creatures and leaves a Treasure behind.
    #[test]
    fn prizefight_fights_and_makes_treasure() {
        let mut g = two_player_game();
        let mine = g.add_card_to_battlefield(0, catalog::hill_giant()); // 3/3
        let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
        let ctx = EffectContext {
            targets: vec![Target::Permanent(mine), Target::Permanent(theirs)],
            ..EffectContext::for_spell(0, None, 0, 0)
        };
        g.resolve_effect(&catalog::prizefight().effect.clone(), &ctx).unwrap();
        drain_stack(&mut g);
        assert!(!g.battlefield.iter().any(|c| c.id == theirs), "2/2 dies to the 3/3");
        assert_eq!(
            g.battlefield.iter().filter(|c| c.controller == 0 && c.definition.name == "Treasure").count(),
            1,
            "a Treasure is minted"
        );
    }

    /// Out Cold can't be countered, taps + stuns up to two creatures, and
    /// investigates.
    #[test]
    fn out_cold_taps_stuns_and_investigates() {
        let mut g = two_player_game();
        assert!(catalog::out_cold().keywords.contains(&Keyword::CantBeCountered));
        let a = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let b = g.add_card_to_battlefield(1, catalog::hill_giant());
        let ctx = EffectContext {
            targets: vec![Target::Permanent(a), Target::Permanent(b)],
            ..EffectContext::for_spell(0, None, 0, 0)
        };
        g.resolve_effect(&catalog::out_cold().effect.clone(), &ctx).unwrap();
        for id in [a, b] {
            assert!(g.battlefield_find(id).unwrap().tapped, "creature tapped");
            assert_eq!(g.battlefield_find(id).unwrap().counter_count(CounterType::Stun), 1, "stun counter");
        }
        assert_eq!(
            g.battlefield.iter().filter(|c| c.controller == 0 && c.definition.name == "Clue").count(),
            1,
            "a Clue is investigated"
        );
    }

    /// Harvester of Misery's ETB shrinks every other creature but not itself.
    #[test]
    fn harvester_etb_shrinks_others() {
        let mut g = two_player_game();
        let harv = g.add_card_to_battlefield(0, catalog::harvester_of_misery());
        let other = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let enemy = g.add_card_to_battlefield(1, catalog::hill_giant()); // 3/3
        let effect = catalog::harvester_of_misery().triggered_abilities[0].effect.clone();
        g.resolve_effect(&effect, &EffectContext::for_trigger(harv, 0, None, 0)).unwrap();
        g.check_state_based_actions();
        assert!(!g.battlefield.iter().any(|c| c.id == other), "the 2/2 dies to -2/-2");
        assert_eq!(g.computed_permanent(enemy).unwrap().toughness, 1, "3/3 → 1/1");
        assert_eq!(g.computed_permanent(harv).unwrap().power, 5, "Harvester unaffected");
    }

    /// Harvester of Misery's from-hand ability: {1}{B}, Discard this card: target
    /// creature gets -2/-2. Activating discards Harvester and shrinks the target.
    #[test]
    fn harvester_hand_ability_shrinks_target() {
        let mut g = two_player_game();
        let enemy = g.add_card_to_battlefield(1, catalog::hill_giant()); // 3/3
        let harv = g.add_card_to_hand(0, catalog::harvester_of_misery());
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g.players[0].mana_pool.add(crabomination::mana::Color::Black, 1);
        g.players[0].mana_pool.add_colorless(1);
        g.perform_action(GameAction::ActivateAbility {
            card_id: harv,
            ability_index: 0,
            target: Some(Target::Permanent(enemy)),
            additional_targets: Vec::new(),
            x_value: None,
        })
        .expect("from-hand discard ability");
        drain_stack(&mut g);
        assert_eq!(g.computed_permanent(enemy).unwrap().toughness, 1, "3/3 → 1/1");
        assert!(g.players[0].graveyard.iter().any(|c| c.id == harv), "Harvester discarded as a cost");
    }

    /// Krovod Haunch buffs its wearer +2/+0 and, when it dies, its owner may pay
    /// {1}{W} to make two Dogs.
    #[test]
    fn krovod_haunch_buffs_and_makes_dogs() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let haunch = g.add_card_to_battlefield(0, catalog::krovod_haunch());
        g.battlefield_find_mut(haunch).unwrap().attached_to = Some(bear);
        assert_eq!(g.computed_permanent(bear).unwrap().power, 4, "2 + 2 = 4");

        // Resolve the death-trigger body with mana available and a yes-decider.
        g.players[0].mana_pool.add(crabomination::mana::Color::White, 1);
        g.players[0].mana_pool.add_colorless(1);
        let effect = catalog::krovod_haunch().triggered_abilities[0].effect.clone();
        g.decider = Box::new(crabomination::decision::ScriptedDecider::new([
            crabomination::decision::DecisionAnswer::Bool(true),
        ]));
        g.resolve_effect(&effect, &EffectContext::for_trigger(haunch, 0, None, 0)).unwrap();
        drain_stack(&mut g);
        assert_eq!(
            g.battlefield.iter().filter(|c| c.controller == 0 && c.definition.name == "Dog").count(),
            2,
            "two Dogs created"
        );
    }
}

mod recent230 {
    use crabomination::card::{CounterType, CreatureType, Keyword};
    use crabomination::catalog;
    use crabomination::game::effects::EffectContext;
    use crabomination::game::types::Target;
    use crabomination::game::{drain_stack, two_player_game};

    /// Wickerfolk Thresher digs the top land onto the battlefield when delirium is
    /// active.
    #[test]
    fn wickerfolk_thresher_digs_a_land_with_delirium() {
        let mut g = two_player_game();
        let thresher = g.add_card_to_battlefield(0, catalog::wickerfolk_thresher());
        // Put a land on top of the library.
        let land_id = crabomination::card::CardId(9001);
        g.players[0].add_to_library_top(land_id, catalog::forest());
        let before = g.battlefield.iter().filter(|c| c.controller == 0 && c.definition.name == "Forest").count();
        let effect = catalog::wickerfolk_thresher().triggered_abilities[0].effect.clone();
        g.resolve_effect(&effect, &EffectContext::for_trigger(thresher, 0, None, 0)).unwrap();
        drain_stack(&mut g);
        let after = g.battlefield.iter().filter(|c| c.controller == 0 && c.definition.name == "Forest").count();
        assert_eq!(after, before + 1, "the top land is put onto the battlefield");
    }

    /// Resilient Roadrunner has haste and protection from Coyotes; its {3} makes it
    /// blockable only by haste creatures.
    #[test]
    fn resilient_roadrunner_evasion() {
        let mut g = two_player_game();
        let rr = g.add_card_to_battlefield(0, catalog::resilient_roadrunner());
        let cp = g.computed_permanent(rr).unwrap();
        assert!(cp.keywords.contains(&Keyword::Haste));
        assert!(cp.keywords.contains(&Keyword::ProtectionFromCreatureType(CreatureType::Coyote)));
        let effect = catalog::resilient_roadrunner().activated_abilities[0].effect.clone();
        g.resolve_effect(&effect, &EffectContext::for_ability(rr, 0, None)).unwrap();
        assert!(
            g.computed_permanent(rr)
                .unwrap()
                .keywords
                .iter()
                .any(|k| matches!(k, Keyword::CantBeBlockedExceptBy(_))),
            "gains can't-be-blocked-except-by-haste"
        );
    }

    /// Giant Beaver's saddled-attack trigger puts a +1/+1 counter on a creature you
    /// control.
    #[test]
    fn giant_beaver_counter_when_saddled() {
        let mut g = two_player_game();
        let beaver = g.add_card_to_battlefield(0, catalog::giant_beaver());
        let ally = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let effect = catalog::giant_beaver().triggered_abilities[0].effect.clone();
        let ctx = EffectContext {
            targets: vec![Target::Permanent(ally)],
            ..EffectContext::for_trigger(beaver, 0, None, 0)
        };
        g.resolve_effect(&effect, &ctx).unwrap();
        assert_eq!(g.battlefield_find(ally).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
        assert_eq!(catalog::giant_beaver().keywords.iter().filter(|k| matches!(k, Keyword::Saddle(3))).count(), 1);
    }

    /// Ornery Tumblewagg's saddled-attack trigger doubles the +1/+1 counters on a
    /// target creature.
    #[test]
    fn ornery_tumblewagg_doubles_counters() {
        let mut g = two_player_game();
        let wagg = g.add_card_to_battlefield(0, catalog::ornery_tumblewagg());
        let ally = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.battlefield_find_mut(ally).unwrap().add_counters(CounterType::PlusOnePlusOne, 3);
        // second ability = the saddled-attack doubler
        let effect = catalog::ornery_tumblewagg().triggered_abilities[1].effect.clone();
        let ctx = EffectContext {
            targets: vec![Target::Permanent(ally)],
            ..EffectContext::for_trigger(wagg, 0, None, 0)
        };
        g.resolve_effect(&effect, &ctx).unwrap();
        assert_eq!(g.battlefield_find(ally).unwrap().counter_count(CounterType::PlusOnePlusOne), 6, "3 → 6");
    }
}

mod recent231 {
    use crabomination::card::CounterType;
    use crabomination::catalog;
    
    use crabomination::game::effects::EffectContext;
    use crabomination::game::types::Target;
    use crabomination::game::{drain_stack, two_player_game};

    /// Volcanic Spite deals 3 to a creature; the optional loot is declined cleanly.
    #[test]
    fn volcanic_spite_burns_a_creature() {
        let mut g = two_player_game();
        let enemy = g.add_card_to_battlefield(1, catalog::hill_giant()); // 3/3
        let ctx = EffectContext {
            targets: vec![Target::Permanent(enemy)],
            ..EffectContext::for_spell(0, None, 0, 0)
        };
        // AutoDecider declines the optional bottom-then-draw.
        g.resolve_effect(&catalog::volcanic_spite().effect.clone(), &ctx).unwrap();
        drain_stack(&mut g);
        assert!(!g.battlefield.iter().any(|c| c.id == enemy), "3/3 dies to 3 damage");
    }

    /// Rampaging Soulrager is 1/4 with no unlocked doors and 4/4 once two doors
    /// among your Rooms are unlocked.
    #[test]
    fn rampaging_soulrager_grows_with_two_doors() {
        let mut g = two_player_game();
        let sr = g.add_card_to_battlefield(0, catalog::rampaging_soulrager());
        assert_eq!(g.computed_permanent(sr).unwrap().power, 1, "1/4 with no doors");
        let room = g.add_card_to_battlefield(0, catalog::roaring_furnace_steaming_sauna());
        g.battlefield_find_mut(room).unwrap().unlock_room_door(false);
        g.battlefield_find_mut(room).unwrap().unlock_room_door(true);
        assert_eq!(g.computed_permanent(sr).unwrap().power, 4, "+3/+0 with two unlocked doors");
    }

    /// Lilysplash Mentor blinks your creature and returns it with a +1/+1 counter.
    #[test]
    fn lilysplash_mentor_blinks_with_counter() {
        let mut g = two_player_game();
        let mentor = g.add_card_to_battlefield(0, catalog::lilysplash_mentor());
        let ally = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let effect = catalog::lilysplash_mentor().activated_abilities[0].effect.clone();
        let ctx = EffectContext {
            targets: vec![Target::Permanent(ally)],
            ..EffectContext::for_ability(mentor, 0, None)
        };
        g.resolve_effect(&effect, &ctx).unwrap();
        drain_stack(&mut g);
        // The original object left; a fresh Grizzly Bears is back with a counter.
        let returned = g
            .battlefield
            .iter()
            .find(|c| c.controller == 0 && c.definition.name == "Grizzly Bears")
            .expect("creature returned to the battlefield");
        assert_eq!(returned.counter_count(CounterType::PlusOnePlusOne), 1, "returns with a +1/+1 counter");
    }
}

mod recent232 {
    use crabomination::card::{CardType, Keyword};
    use crabomination::catalog;
    
    use crabomination::game::effects::EffectContext;
    use crabomination::game::two_player_game;

    /// Haunted Screen taps for W or B.
    #[test]
    fn haunted_screen_taps_for_wb() {
        use crabomination::mana::Color;
        let mut g = two_player_game();
        let screen = g.add_card_to_battlefield(0, catalog::haunted_screen());
        let effect = catalog::haunted_screen().activated_abilities[0].effect.clone();
        g.resolve_effect(&effect, &EffectContext::for_ability(screen, 0, None)).unwrap();
        let pool = g.players[0].mana_pool.total();
        assert_eq!(pool, 1, "one mana produced");
        assert!(
            g.players[0].mana_pool.amount(Color::White) == 1 || g.players[0].mana_pool.amount(Color::Black) == 1,
            "the mana is white or black",
        );
    }

    /// Fear of Infinity is a flying, lifelink Nightmare that can't block.
    #[test]
    fn fear_of_infinity_keywords() {
        let def = catalog::fear_of_infinity();
        assert!(def.keywords.contains(&Keyword::Flying));
        assert!(def.keywords.contains(&Keyword::Lifelink));
        assert!(def.keywords.contains(&Keyword::CantBlock));
        assert!(def.card_types.contains(&CardType::Enchantment) && def.card_types.contains(&CardType::Creature));
    }
}

mod recent233 {
    use crabomination::catalog;
    use crabomination::effect::{Effect, SpreeMode};
    use crabomination::game::effects::EffectContext;
    use crabomination::game::types::Target;
    use crabomination::game::two_player_game;

    fn spree_modes(def: &crabomination::card::CardDefinition) -> Vec<SpreeMode> {
        match &def.effect {
            Effect::Spree { modes } => modes.clone(),
            _ => panic!("not a spree card"),
        }
    }

    /// Metamorphic Blast's shrink mode turns a creature into a 0/1 Rabbit.
    #[test]
    fn metamorphic_blast_shrinks_to_rabbit() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(1, catalog::hill_giant()); // 3/3
        let modes = spree_modes(&catalog::metamorphic_blast());
        let ctx = EffectContext {
            targets: vec![Target::Permanent(bear)],
            ..EffectContext::for_spell(0, None, 0, 0)
        };
        g.resolve_effect(&modes[0].effect, &ctx).unwrap();
        let cp = g.computed_permanent(bear).unwrap();
        assert_eq!((cp.power, cp.toughness), (0, 1), "becomes a 0/1");
        assert!(cp.subtypes.creature_types.contains(&crabomination::card::CreatureType::Rabbit), "a Rabbit");
    }

    /// Return the Favor's copy mode duplicates an instant/sorcery spell on the
    /// stack (the copy lands on the stack above the original).
    #[test]
    fn return_the_favor_copies_a_spell() {
        let mut g = two_player_game();
        // Put a Lightning Bolt on the stack targeting a creature.
        let target = g.add_card_to_battlefield(1, catalog::serra_angel());
        let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
        g.players[0].mana_pool.add(crabomination::mana::Color::Red, 1);
        g.step = crabomination::game::TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g.perform_action(crabomination::game::GameAction::CastSpell {
            card_id: bolt,
            target: Some(Target::Permanent(target)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .unwrap();
        let stack_before = g.stack.len();
        let modes = spree_modes(&catalog::return_the_favor());
        // The spell on the stack is targeted by its own card id.
        let ctx = EffectContext {
            targets: vec![Target::Permanent(bolt)],
            ..EffectContext::for_spell(0, None, 0, 0)
        };
        g.resolve_effect(&modes[0].effect, &ctx).unwrap();
        assert!(g.stack.len() > stack_before, "a copy was added to the stack");
    }
}

mod recent234 {
    use crabomination::card::{CounterType, Keyword};
    use crabomination::catalog;
    use crabomination::effect::{Effect, SpreeMode};
    use crabomination::game::effects::EffectContext;
    use crabomination::game::types::Target;
    use crabomination::game::{drain_stack, two_player_game};

    fn spree_modes(def: &crabomination::card::CardDefinition) -> Vec<SpreeMode> {
        match &def.effect {
            Effect::Spree { modes } => modes.clone(),
            _ => panic!("not a spree card"),
        }
    }

    /// Trash the Town's counter mode adds two +1/+1 counters.
    #[test]
    fn trash_the_town_adds_counters() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let modes = spree_modes(&catalog::trash_the_town());
        let ctx = EffectContext { targets: vec![Target::Permanent(bear)], ..EffectContext::for_spell(0, None, 0, 0) };
        g.resolve_effect(&modes[0].effect, &ctx).unwrap();
        assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne), 2);
    }

    /// Unfortunate Accident's first mode destroys a creature; its second makes a
    /// Mercenary token.
    #[test]
    fn unfortunate_accident_modes() {
        let mut g = two_player_game();
        let enemy = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let modes = spree_modes(&catalog::unfortunate_accident());
        let ctx = EffectContext { targets: vec![Target::Permanent(enemy)], ..EffectContext::for_spell(0, None, 0, 0) };
        g.resolve_effect(&modes[0].effect, &ctx).unwrap();
        drain_stack(&mut g);
        assert!(!g.battlefield.iter().any(|c| c.id == enemy), "creature destroyed");

        g.resolve_effect(&modes[1].effect, &EffectContext::for_spell(0, None, 0, 0)).unwrap();
        assert_eq!(
            g.battlefield.iter().filter(|c| c.controller == 0 && c.definition.name == "Mercenary").count(),
            1,
            "a Mercenary token appears",
        );
    }

    /// Thunder Lasso attaches on ETB and grants +1/+1.
    #[test]
    fn thunder_lasso_attaches_and_buffs() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
        let lasso = g.add_card_to_battlefield(0, catalog::thunder_lasso());
        let effect = catalog::thunder_lasso().triggered_abilities[0].effect.clone();
        let ctx = EffectContext { targets: vec![Target::Permanent(bear)], ..EffectContext::for_trigger(lasso, 0, None, 0) };
        g.resolve_effect(&effect, &ctx).unwrap();
        assert_eq!(g.battlefield_find(lasso).unwrap().attached_to, Some(bear), "attached to the bear");
        assert_eq!(g.computed_permanent(bear).unwrap().power, 3, "2 + 1 = 3");
        assert!(catalog::thunder_lasso().keywords.iter().any(|k| matches!(k, Keyword::Equip(_))));
    }
}

mod recent235 {
    use crabomination::card::CounterType;
    use crabomination::catalog;
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    use crabomination::effect::Effect;
    use crabomination::game::effects::EffectContext;
    use crabomination::game::types::Target;
    use crabomination::game::{drain_stack, two_player_game};

    /// A Room's back-door unlock trigger effect, by door name.
    fn door_effect(def: &crabomination::card::CardDefinition, right: bool) -> Effect {
        let room = def.room.as_ref().expect("room card");
        let door = if right { &room.right } else { &room.left };
        door.triggered_abilities[0].effect.clone()
    }

    /// Surgical Suite's unlock reanimates a creature MV≤3 from the graveyard.
    #[test]
    fn surgical_suite_reanimates() {
        let mut g = two_player_game();
        let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
        let def = catalog::surgical_suite_hospital_room();
        let src = g.add_card_to_battlefield(0, catalog::surgical_suite_hospital_room());
        let ctx = EffectContext {
            targets: vec![Target::Permanent(bear)],
            ..EffectContext::for_trigger(src, 0, None, 0)
        };
        g.resolve_effect(&door_effect(&def, false), &ctx).unwrap();
        drain_stack(&mut g);
        assert!(g.battlefield.iter().any(|c| c.id == bear && c.controller == 0), "bear reanimated");
    }

    /// Slimy Aquarium manifests a face-down 2/2 and puts a +1/+1 counter on it,
    /// exercising the manifest-dread `LastMoved` rider.
    #[test]
    fn slimy_aquarium_manifests_and_counters() {
        let mut g = two_player_game();
        // Ensure the library has cards to manifest.
        for _ in 0..3 {
            g.add_card_to_library(0, catalog::grizzly_bears());
        }
        let def = catalog::underwater_tunnel_slimy_aquarium();
        let src = g.add_card_to_battlefield(0, catalog::underwater_tunnel_slimy_aquarium());
        let ctx = EffectContext::for_trigger(src, 0, None, 0);
        g.resolve_effect(&door_effect(&def, true), &ctx).unwrap();
        // A face-down 2/2 with a +1/+1 counter now sits on the battlefield.
        let manifested = g
            .battlefield
            .iter()
            .find(|c| c.controller == 0 && c.face_down && c.id != src)
            .expect("a manifested creature");
        assert_eq!(manifested.counter_count(CounterType::PlusOnePlusOne), 1, "counter placed");
    }

    /// Moldering Gym searches a basic land onto the battlefield tapped.
    #[test]
    fn moldering_gym_fetches_basic() {
        let mut g = two_player_game();
        let forest = g.add_card_to_library(0, catalog::forest());
        let def = catalog::moldering_gym_weight_room();
        let src = g.add_card_to_battlefield(0, catalog::moldering_gym_weight_room());
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(forest))]));
        let ctx = EffectContext::for_trigger(src, 0, None, 0);
        g.resolve_effect(&door_effect(&def, false), &ctx).unwrap();
        drain_stack(&mut g);
        assert!(
            g.battlefield.iter().any(|c| c.controller == 0 && c.definition.is_land() && c.tapped),
            "a basic land entered tapped",
        );
    }

    /// Greenhouse grants every land you control a "{T}: Add any color" ability.
    #[test]
    fn greenhouse_grants_land_mana_ability() {
        let def = catalog::greenhouse_rickety_gazebo();
        let room = def.room.as_ref().unwrap();
        assert!(
            !room.left.static_abilities.is_empty(),
            "Greenhouse has the land-grant static",
        );
    }

    /// Rickety Gazebo mills four, then returns up to two permanent cards to hand.
    #[test]
    fn rickety_gazebo_mill_then_return() {
        let mut g = two_player_game();
        let ids: Vec<_> = (0..4).map(|_| g.add_card_to_library(0, catalog::grizzly_bears())).collect();
        let def = catalog::greenhouse_rickety_gazebo();
        let src = g.add_card_to_battlefield(0, catalog::greenhouse_rickety_gazebo());
        // Pick two of the four milled creatures to return.
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Cards(vec![ids[0], ids[1]])]));
        let ctx = EffectContext::for_trigger(src, 0, None, 0);
        g.resolve_effect(&door_effect(&def, true), &ctx).unwrap();
        let in_hand = g.players[0].hand.iter().filter(|c| ids.contains(&c.id)).count();
        let in_gy = g.players[0].graveyard.iter().filter(|c| ids.contains(&c.id)).count();
        assert_eq!(in_hand, 2, "two returned to hand");
        assert_eq!(in_gy, 2, "the other two stayed milled");
    }

    /// Walk-In Closet grants "play lands from your graveyard".
    #[test]
    fn walk_in_closet_static() {
        let def = catalog::walk_in_closet_forgotten_cellar();
        let room = def.room.as_ref().unwrap();
        assert!(!room.left.static_abilities.is_empty(), "Walk-In Closet carries a static");
    }

    /// Orphans of the Wheat taps chosen creatures on attack and pumps per tap.
    #[test]
    fn orphans_taps_and_pumps() {
        let mut g = two_player_game();
        let orphans = g.add_card_to_battlefield(0, catalog::orphans_of_the_wheat()); // 2/1
        let a = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let b = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Cards(vec![a, b])]));
        let effect = catalog::orphans_of_the_wheat().triggered_abilities[0].effect.clone();
        let ctx = EffectContext::for_trigger(orphans, 0, None, 0);
        g.resolve_effect(&effect, &ctx).unwrap();
        assert!(g.battlefield_find(a).unwrap().tapped && g.battlefield_find(b).unwrap().tapped, "both tapped");
        assert_eq!(g.computed_permanent(orphans).unwrap().power, 4, "2 + 2 tapped = 4");
    }
}

mod recent236 {
    use crabomination::card::{CardType, CounterType, Keyword};
    use crabomination::catalog;
    use crabomination::effect::Effect;
    use crabomination::game::effects::EffectContext;
    use crabomination::game::types::Target;
    use crabomination::game::{drain_stack, two_player_game};

    fn door_effect(def: &crabomination::card::CardDefinition, right: bool) -> Effect {
        let room = def.room.as_ref().expect("room card");
        let door = if right { &room.right } else { &room.left };
        door.triggered_abilities[0].effect.clone()
    }

    /// Grand Entryway's unlock makes a 1/1 Glimmer enchantment creature.
    #[test]
    fn grand_entryway_makes_glimmer() {
        let mut g = two_player_game();
        let def = catalog::grand_entryway_elegant_rotunda();
        let src = g.add_card_to_battlefield(0, catalog::grand_entryway_elegant_rotunda());
        let ctx = EffectContext::for_trigger(src, 0, None, 0);
        g.resolve_effect(&door_effect(&def, false), &ctx).unwrap();
        let glimmer = g
            .battlefield
            .iter()
            .find(|c| c.definition.name == "Glimmer" && c.controller == 0)
            .expect("Glimmer token");
        assert!(glimmer.definition.card_types.contains(&CardType::Enchantment), "enchantment creature");
    }

    /// Elegant Rotunda puts a +1/+1 counter on each of up to two target creatures.
    #[test]
    fn elegant_rotunda_counters_two() {
        let mut g = two_player_game();
        let a = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let b = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let def = catalog::grand_entryway_elegant_rotunda();
        let src = g.add_card_to_battlefield(0, catalog::grand_entryway_elegant_rotunda());
        let ctx = EffectContext {
            targets: vec![Target::Permanent(a), Target::Permanent(b)],
            ..EffectContext::for_trigger(src, 0, None, 0)
        };
        g.resolve_effect(&door_effect(&def, true), &ctx).unwrap();
        assert_eq!(g.battlefield_find(a).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
        assert_eq!(g.battlefield_find(b).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
    }

    /// Derelict Attic draws two and loses 2 life on unlock.
    #[test]
    fn derelict_attic_draw_lose() {
        let mut g = two_player_game();
        for _ in 0..2 {
            g.add_card_to_library(0, catalog::grizzly_bears());
        }
        let def = catalog::derelict_attic_widows_walk();
        let src = g.add_card_to_battlefield(0, catalog::derelict_attic_widows_walk());
        let hand = g.players[0].hand.len();
        let life = g.players[0].life;
        let ctx = EffectContext::for_trigger(src, 0, None, 0);
        g.resolve_effect(&door_effect(&def, false), &ctx).unwrap();
        assert_eq!(g.players[0].hand.len(), hand + 2, "drew two");
        assert_eq!(g.players[0].life, life - 2, "lost 2 life");
    }

    /// Funeral Room drains 1 whenever a creature you control dies.
    #[test]
    fn funeral_room_drains_on_death() {
        let mut g = two_player_game();
        let room = g.add_card_to_battlefield(0, catalog::funeral_room_awakening_hall());
        // The Funeral Room (left) door's death drain is live only once unlocked.
        g.battlefield_find_mut(room).unwrap().unlock_room_door(false);
        let victim = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let opp_life = g.players[1].life;
        let my_life = g.players[0].life;
        let mut evs = g.remove_to_graveyard_with_triggers(victim);
        evs.push(crabomination::game::GameEvent::CreatureDied { card_id: victim });
        g.dispatch_triggers_for_events(&evs);
        drain_stack(&mut g);
        assert_eq!(g.players[1].life, opp_life - 1, "opponent lost 1");
        assert_eq!(g.players[0].life, my_life + 1, "you gained 1");
    }

    /// Awakening Hall returns every creature card from your graveyard.
    #[test]
    fn awakening_hall_mass_reanimate() {
        let mut g = two_player_game();
        let a = g.add_card_to_graveyard(0, catalog::grizzly_bears());
        let b = g.add_card_to_graveyard(0, catalog::grizzly_bears());
        let def = catalog::funeral_room_awakening_hall();
        let src = g.add_card_to_battlefield(0, catalog::funeral_room_awakening_hall());
        let ctx = EffectContext::for_trigger(src, 0, None, 0);
        g.resolve_effect(&door_effect(&def, true), &ctx).unwrap();
        drain_stack(&mut g);
        assert!(g.battlefield.iter().any(|c| c.id == a), "first creature reanimated");
        assert!(g.battlefield.iter().any(|c| c.id == b), "second creature reanimated");
    }

    /// Defaced Gallery pumps attacking creatures when you attack.
    #[test]
    fn defaced_gallery_pumps_attackers() {
        let mut g = two_player_game();
        let def = catalog::painters_studio_defaced_gallery();
        let src = g.add_card_to_battlefield(0, catalog::painters_studio_defaced_gallery());
        let attacker = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.set_attacking(vec![crabomination::game::types::Attack {
            attacker,
            target: crabomination::game::types::AttackTarget::Player(1),
        }]);
        let ctx = EffectContext::for_trigger(src, 0, None, 0);
        g.resolve_effect(&door_effect(&def, true), &ctx).unwrap();
        assert_eq!(g.computed_permanent(attacker).unwrap().power, 3, "2 + 1 = 3");
    }

    /// Terror of Towashi's attack rider reanimates when the cost is paid.
    #[test]
    fn terror_of_towashi_reanimates_on_pay() {
        let mut g = two_player_game();
        let dead = g.add_card_to_graveyard(0, catalog::grizzly_bears());
        let terror = g.add_card_to_battlefield(0, catalog::terror_of_towashi());
        g.players[0].mana_pool.add(crabomination::mana::Color::Black, 1);
        g.players[0].mana_pool.add_colorless(3);
        // Say yes to the {3}{B} may-pay, then target the graveyard creature.
        g.decider = Box::new(crabomination::decision::ScriptedDecider::new([
            crabomination::decision::DecisionAnswer::Bool(true),
        ]));
        let effect = catalog::terror_of_towashi().triggered_abilities[0].effect.clone();
        let ctx = EffectContext::for_trigger(terror, 0, None, 0);
        g.resolve_effect(&effect, &ctx).unwrap();
        drain_stack(&mut g);
        assert!(g.battlefield.iter().any(|c| c.id == dead), "creature reanimated after paying");
        // Deathtouch is printed.
        assert!(catalog::terror_of_towashi().keywords.contains(&Keyword::Deathtouch));
    }
}

mod recent237 {
    use crabomination::card::Keyword;
    use crabomination::catalog;
    use crabomination::effect::Effect;
    use crabomination::game::effects::EffectContext;
    use crabomination::game::types::Target;
    use crabomination::game::{drain_stack, two_player_game};

    fn door_effect(def: &crabomination::card::CardDefinition, right: bool) -> Effect {
        let room = def.room.as_ref().expect("room card");
        let door = if right { &room.right } else { &room.left };
        door.triggered_abilities[0].effect.clone()
    }

    /// Restricted Office's unlock destroys all creatures with power 3+.
    #[test]
    fn restricted_office_wraths_big_creatures() {
        let mut g = two_player_game();
        let big = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2 — safe
        let bigger = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4 flying
        let def = catalog::restricted_office_lecture_hall();
        let src = g.add_card_to_battlefield(0, catalog::restricted_office_lecture_hall());
        let ctx = EffectContext::for_trigger(src, 0, None, 0);
        g.resolve_effect(&door_effect(&def, false), &ctx).unwrap();
        drain_stack(&mut g);
        assert!(g.battlefield.iter().any(|c| c.id == big), "2/2 survives");
        assert!(!g.battlefield.iter().any(|c| c.id == bigger), "4/4 destroyed");
    }

    /// Tunnel of Hate grants double strike to a target attacker when you attack.
    #[test]
    fn tunnel_of_hate_grants_double_strike() {
        let mut g = two_player_game();
        let attacker = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.set_attacking(vec![crabomination::game::types::Attack {
            attacker,
            target: crabomination::game::types::AttackTarget::Player(1),
        }]);
        let def = catalog::ticket_booth_tunnel_of_hate();
        let src = g.add_card_to_battlefield(0, catalog::ticket_booth_tunnel_of_hate());
        let ctx = EffectContext {
            targets: vec![Target::Permanent(attacker)],
            ..EffectContext::for_trigger(src, 0, None, 0)
        };
        g.resolve_effect(&door_effect(&def, true), &ctx).unwrap();
        assert!(
            g.computed_permanent(attacker).unwrap().keywords.contains(&Keyword::DoubleStrike),
            "gained double strike",
        );
    }

    /// Peer Past the Veil discards the hand, then draws one per graveyard card type.
    #[test]
    fn peer_past_the_veil_draws_by_types() {
        let mut g = two_player_game();
        // Graveyard has a creature and an instant → 2 card types.
        g.add_card_to_graveyard(0, catalog::grizzly_bears());
        g.add_card_to_graveyard(0, catalog::lightning_bolt());
        // Hand has three cards; library has plenty to draw.
        g.add_card_to_hand(0, catalog::grizzly_bears());
        g.add_card_to_hand(0, catalog::grizzly_bears());
        for _ in 0..6 {
            g.add_card_to_library(0, catalog::forest());
        }
        g.resolve_effect(&catalog::peer_past_the_veil().effect, &EffectContext::for_spell(0, None, 0, 0))
            .unwrap();
        // Discarded the hand (those cards add types), then drew types-in-gy.
        // GY now holds creature + instant + the discarded creatures → still 2 types.
        assert_eq!(g.players[0].hand.len(), 2, "drew 2 (creature + instant card types)");
        assert!(g.players[0].hand.iter().all(|c| c.definition.is_land()), "drew from library");
    }

    /// The Swarmweaver makes two Insects and, with delirium, buffs them.
    #[test]
    fn swarmweaver_tokens_and_delirium_anthem() {
        let mut g = two_player_game();
        let sw = g.add_card_to_battlefield(0, catalog::the_swarmweaver());
        let effect = catalog::the_swarmweaver().triggered_abilities[0].effect.clone();
        g.resolve_effect(&effect, &EffectContext::for_trigger(sw, 0, None, 0)).unwrap();
        let insects: Vec<_> = g
            .battlefield
            .iter()
            .filter(|c| c.controller == 0 && c.definition.name == "Insect")
            .map(|c| c.id)
            .collect();
        assert_eq!(insects.len(), 2, "two Insect tokens");
        // No delirium yet → 1/1.
        assert_eq!(g.computed_permanent(insects[0]).unwrap().power, 1, "1/1 without delirium");
        // Seed 4 card types in graveyard → delirium.
        g.add_card_to_graveyard(0, catalog::grizzly_bears()); // creature
        g.add_card_to_graveyard(0, catalog::lightning_bolt()); // instant
        g.add_card_to_graveyard(0, catalog::forest()); // land
        g.add_card_to_graveyard(0, catalog::the_swarmweaver()); // artifact
        assert_eq!(g.computed_permanent(insects[0]).unwrap().power, 2, "2/2 with delirium");
        assert!(
            g.computed_permanent(insects[0]).unwrap().keywords.contains(&Keyword::Deathtouch),
            "deathtouch with delirium",
        );
    }
}
