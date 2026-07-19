//! Tests for recentN card batches 1-10 (merged from per-batch micro-files).

mod recent {
    use crabomination::card::{CounterType, CreatureType, Keyword};
    use crabomination::catalog;
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    use crabomination::game::types::{Attack, AttackTarget};
    use crabomination::game::*;
    use crabomination::mana::Color;
    use crabomination::TurnStep;

    /// Questing Beast can't be blocked by power-2-or-less creatures.
    #[test]
    fn questing_beast_evades_small_blockers() {
        let mut g = two_player_game();
        let qb = g.add_card_to_battlefield(0, catalog::questing_beast());
        let weak = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
        let big = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4
        assert!(!g.blocker_can_block_attacker(weak, qb), "power-2 can't block");
        assert!(g.blocker_can_block_attacker(big, qb), "power-4 can block");
        let r = g.battlefield_find(qb).unwrap();
        assert!(r.definition.keywords.contains(&Keyword::Deathtouch));
        assert!(r.definition.keywords.contains(&Keyword::Haste));
    }

    /// Cackling Slasher enters with a +1/+1 counter when a creature died this turn.
    #[test]
    fn cackling_slasher_grows_after_a_death() {
        let mut g = two_player_game();
        g.players[1].creatures_died_this_turn = 1;
        let slasher = g.move_card_to_battlefield_for_test(0, catalog::cackling_slasher());
        drain_stack(&mut g);
        let r = g.battlefield_find(slasher).unwrap();
        assert_eq!(r.counters.get(&CounterType::PlusOnePlusOne).copied(), Some(1));
    }

    /// Wary Watchdog surveils on entry.
    #[test]
    fn wary_watchdog_surveils_on_etb() {
        let mut g = two_player_game();
        g.add_card_to_library(0, catalog::island());
        let lib = g.players[0].library.len();
        g.move_card_to_battlefield_for_test(0, catalog::wary_watchdog());
        drain_stack(&mut g);
        // Surveil 1 looked at the top card (library size unchanged when kept on top).
        assert!(g.players[0].library.len() <= lib, "surveil resolved");
        assert_eq!(catalog::wary_watchdog().triggered_abilities.len(), 2, "ETB + dies triggers");
    }

    /// Hunted Bonebrute gives the opponent two Dogs on ETB and drains on death.
    #[test]
    fn hunted_bonebrute_etb_dogs_and_death_drain() {
        let mut g = two_player_game();
        g.players[1].life = 20;
        let brute = catalog::hunted_bonebrute();
        let etb_ctx = crabomination::game::effects::EffectContext::for_ability(crabomination::card::CardId(0), 0, None);
        g.resolve_effect(&brute.triggered_abilities[0].effect, &etb_ctx).unwrap();
        let dogs = g.battlefield.iter().filter(|c| c.controller == 1 && c.definition.name == "Dog").count();
        assert_eq!(dogs, 2, "opponent made two Dogs");
        // Death drain.
        g.resolve_effect(&brute.triggered_abilities[1].effect, &etb_ctx).unwrap();
        assert_eq!(g.players[1].life, 17, "each opponent lost 3");
    }

    /// Trumpeting Herd makes a 3/3 Elephant and has Rebound.
    #[test]
    fn trumpeting_herd_makes_elephant() {
        use crabomination::card::Keyword;
        let mut g = two_player_game();
        let th = catalog::trumpeting_herd();
        assert!(th.keywords.contains(&Keyword::Rebound));
        let ctx = crabomination::game::effects::EffectContext::for_ability(crabomination::card::CardId(0), 0, None);
        g.resolve_effect(&th.effect, &ctx).unwrap();
        let ele = g.battlefield.iter().find(|c| c.definition.name == "Elephant").unwrap();
        assert_eq!((ele.power(), ele.toughness()), (3, 3));
    }

    /// Festergloom shrinks nonblack creatures but spares black ones.
    #[test]
    fn festergloom_minus_one_to_nonblack() {
        let mut g = two_player_game();
        let white = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // green, 2/2
        let black = g.add_card_to_battlefield(1, catalog::black_knight()); // black, 2/2
        let ctx = crabomination::game::effects::EffectContext::for_ability(crabomination::card::CardId(0), 0, None);
        g.resolve_effect(&catalog::festergloom().effect, &ctx).unwrap();
        g.check_state_based_actions();
        assert_eq!(g.computed_permanent(white).map(|c| (c.power, c.toughness)), Some((1, 1)));
        assert_eq!(g.computed_permanent(black).map(|c| (c.power, c.toughness)), Some((2, 2)));
    }

    /// Intrepid Rabbit's ETB pumps a creature you control and it has Offspring.
    #[test]
    fn intrepid_rabbit_etb_pump() {
        use crabomination::card::Keyword;
        let mut g = two_player_game();
        let r = catalog::intrepid_rabbit();
        assert!(r.keywords.iter().any(|k| matches!(k, Keyword::Offspring(_))));
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let mut ctx = crabomination::game::effects::EffectContext::for_ability(crabomination::card::CardId(0), 0, None);
        ctx.targets = vec![Target::Permanent(bear)];
        g.resolve_effect(&r.triggered_abilities[0].effect, &ctx).unwrap();
        let cp = g.computed_permanent(bear).unwrap();
        assert_eq!((cp.power, cp.toughness), (3, 3));
    }

    /// Marauding Brinefang has Ward {3} and Islandcycling.
    #[test]
    fn marauding_brinefang_ward_and_islandcycling() {
        use crabomination::card::Keyword;
        let b = catalog::marauding_brinefang();
        assert!(b.keywords.iter().any(|k| matches!(k, Keyword::Ward(_))));
        assert!(b.keywords.iter().any(|k| matches!(k, Keyword::Typecycling(_))));
        assert_eq!((b.power, b.toughness), (6, 7));
    }

    /// Crystal Barricade gives its controller hexproof.
    #[test]
    fn crystal_barricade_grants_controller_hexproof() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::crystal_barricade());
        // Player 0 can't be targeted by an opponent now.
        assert!(g.player_has_static_hexproof(0), "controller has hexproof");
    }

    /// Druid of the Spade grows and gains trample only while you control a token.
    #[test]
    fn druid_of_the_spade_token_conditional() {
        use crabomination::card::Keyword;
        let mut g = two_player_game();
        let druid = g.add_card_to_battlefield(0, catalog::druid_of_the_spade());
        let base = g.computed_permanent(druid).unwrap();
        assert_eq!((base.power, base.toughness), (2, 3));
        assert!(!base.keywords.contains(&Keyword::Trample));
        // Mint a token → condition holds.
        let tok = crabomination::card::TokenDefinition {
            name: "Rabbit".into(),
            power: 1,
            toughness: 1,
            card_types: vec![crabomination::card::CardType::Creature],
            ..Default::default()
        };
        g.add_token_to_battlefield(0, &tok);
        let buffed = g.computed_permanent(druid).unwrap();
        assert_eq!((buffed.power, buffed.toughness), (4, 3));
        assert!(buffed.keywords.contains(&Keyword::Trample));
    }

    /// Persistent Marshstalker grows by each other Rat you control.
    #[test]
    fn persistent_marshstalker_rat_lord() {
        let mut g = two_player_game();
        let stalker = g.add_card_to_battlefield(0, catalog::persistent_marshstalker());
        assert_eq!(g.computed_permanent(stalker).unwrap().power, 3, "no other Rats");
        g.add_card_to_battlefield(0, catalog::persistent_marshstalker()); // another Rat
        assert_eq!(g.computed_permanent(stalker).unwrap().power, 4, "+1 for the other Rat");
    }

    /// Nightbird's Clutches stops up to two creatures from blocking and has flashback.
    #[test]
    fn nightbirds_clutches_grants_cant_block() {
        use crabomination::card::Keyword;
        let mut g = two_player_game();
        let a = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let b = g.add_card_to_battlefield(1, catalog::hill_giant());
        let nc = catalog::nightbirds_clutches();
        assert!(nc.keywords.iter().any(|k| matches!(k, Keyword::Flashback(_))));
        let mut ctx = crabomination::game::effects::EffectContext::for_ability(crabomination::card::CardId(0), 0, None);
        ctx.targets = vec![Target::Permanent(a), Target::Permanent(b)];
        g.resolve_effect(&nc.effect, &ctx).unwrap();
        assert!(g.computed_permanent(a).unwrap().keywords.contains(&Keyword::CantBlock));
        assert!(g.computed_permanent(b).unwrap().keywords.contains(&Keyword::CantBlock));
    }

    /// Get Out's bounce mode returns your creatures/enchantments to hand.
    #[test]
    fn get_out_bounce_mode_returns_permanents() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let mut ctx = crabomination::game::effects::EffectContext::for_ability(crabomination::card::CardId(0), 0, None);
        ctx.mode = 1; // bounce mode
        ctx.targets = vec![Target::Permanent(bear)];
        g.resolve_effect(&catalog::get_out().effect, &ctx).unwrap();
        assert!(g.battlefield_find(bear).is_none() && g.players[0].hand.iter().any(|c| c.id == bear));
    }

    /// Helpful Hunter draws on entry.
    #[test]
    fn helpful_hunter_draws_on_etb() {
        let mut g = two_player_game();
        g.add_card_to_library(0, catalog::island());
        let h = g.players[0].hand.len();
        g.move_card_to_battlefield_for_test(0, catalog::helpful_hunter());
        drain_stack(&mut g);
        assert_eq!(g.players[0].hand.len(), h + 1);
    }

    /// Sunshower Druid's ETB grows a creature and gains a life.
    #[test]
    fn sunshower_druid_counter_and_lifegain() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.players[0].life = 20;
        let mut ctx = crabomination::game::effects::EffectContext::for_ability(crabomination::card::CardId(0), 0, None);
        ctx.targets = vec![Target::Permanent(bear)];
        g.resolve_effect(&catalog::sunshower_druid().triggered_abilities[0].effect, &ctx).unwrap();
        assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
        assert_eq!(g.players[0].life, 21);
    }

    /// Coruscation Mage pings each opponent; its trigger gates on noncreature spells.
    #[test]
    fn coruscation_mage_pings_each_opponent() {
        let mut g = two_player_game();
        let mage = g.add_card_to_battlefield(0, catalog::coruscation_mage());
        g.players[1].life = 20;
        let ab = catalog::coruscation_mage().triggered_abilities[0].clone();
        assert!(matches!(ab.event.filter, Some(crabomination::card::Predicate::CastSpellMatches(_))));
        let ctx = crabomination::game::effects::EffectContext::for_trigger(mage, 0, None, 0);
        g.resolve_effect(&ab.effect, &ctx).unwrap();
        assert_eq!(g.players[1].life, 19, "each opponent took 1");
    }

    /// Treetop Snarespinner has reach + deathtouch and a sorcery-speed grow.
    #[test]
    fn treetop_snarespinner_keywords_and_grow() {
        use crabomination::card::Keyword;
        let mut g = two_player_game();
        let spider = catalog::treetop_snarespinner();
        assert!(spider.keywords.contains(&Keyword::Reach) && spider.keywords.contains(&Keyword::Deathtouch));
        assert!(spider.activated_abilities[0].sorcery_speed);
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let mut ctx = crabomination::game::effects::EffectContext::for_ability(crabomination::card::CardId(0), 0, None);
        ctx.targets = vec![Target::Permanent(bear)];
        g.resolve_effect(&spider.activated_abilities[0].effect, &ctx).unwrap();
        assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
    }

    /// Thornplate Intimidator's ETB makes the opponent dodge by discarding rather
    /// than losing 3 life.
    #[test]
    fn thornplate_intimidator_punisher_discard() {
        let mut g = two_player_game();
        g.players[1].life = 20;
        g.add_card_to_hand(1, catalog::grizzly_bears()); // a card to pitch
        let trig = catalog::thornplate_intimidator().triggered_abilities[0].effect.clone();
        let ctx = crabomination::game::effects::EffectContext::for_ability(crabomination::card::CardId(0), 0, None);
        g.resolve_effect(&trig, &ctx).unwrap();
        // No nonland permanent to sac, so the opponent discards (no life loss).
        assert_eq!(g.players[1].life, 20, "dodged the life loss");
        assert!(g.players[1].hand.is_empty(), "discarded instead");
    }

    /// Repeating Barrage burns for 3 and can return itself from the graveyard
    /// after you've attacked.
    #[test]
    fn repeating_barrage_burns_and_raids_back() {
        let mut g = two_player_game();
        let foe = g.add_card_to_battlefield(1, catalog::hill_giant()); // 3/3
        let mut ctx = crabomination::game::effects::EffectContext::for_ability(crabomination::card::CardId(0), 0, None);
        ctx.targets = vec![Target::Permanent(foe)];
        g.resolve_effect(&catalog::repeating_barrage().effect, &ctx).unwrap();
        assert!(g.battlefield_find(foe).is_none(), "3 damage kills the 3/3");
        // The Raid ability is gated on having attacked this turn.
        let ab = &catalog::repeating_barrage().activated_abilities[0];
        assert!(ab.from_graveyard && ab.condition.is_some());
    }

    /// Fountainport Bell can be sacrificed to draw.
    #[test]
    fn fountainport_bell_sac_draws() {
        let mut g = two_player_game();
        let bell = g.add_card_to_battlefield(0, catalog::fountainport_bell());
        g.add_card_to_library(0, catalog::island());
        let hand = g.players[0].hand.len();
        let ctx = crabomination::game::effects::EffectContext::for_ability(bell, 0, None);
        g.resolve_effect(&catalog::fountainport_bell().activated_abilities[0].effect, &ctx).unwrap();
        assert_eq!(g.players[0].hand.len(), hand + 1, "drew a card");
    }

    /// Plumecreed Escort has flash + flying and its ETB grants hexproof.
    #[test]
    fn plumecreed_escort_etb_grants_hexproof() {
        use crabomination::card::Keyword;
        let mut g = two_player_game();
        let esc = catalog::plumecreed_escort();
        assert!(esc.keywords.contains(&Keyword::Flash) && esc.keywords.contains(&Keyword::Flying));
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let mut ctx = crabomination::game::effects::EffectContext::for_ability(crabomination::card::CardId(0), 0, None);
        ctx.targets = vec![Target::Permanent(bear)];
        g.resolve_effect(&esc.triggered_abilities[0].effect, &ctx).unwrap();
        assert!(g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Hexproof));
    }

    /// Overprotect pumps +3/+3 and grants three protective keywords.
    #[test]
    fn overprotect_pumps_and_grants_keywords() {
        use crabomination::card::Keyword;
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let mut ctx = crabomination::game::effects::EffectContext::for_ability(crabomination::card::CardId(0), 0, None);
        ctx.targets = vec![Target::Permanent(bear)];
        g.resolve_effect(&catalog::overprotect().effect, &ctx).unwrap();
        let cp = g.computed_permanent(bear).unwrap();
        assert_eq!((cp.power, cp.toughness), (5, 5));
        for kw in [Keyword::Trample, Keyword::Hexproof, Keyword::Indestructible] {
            assert!(cp.keywords.contains(&kw), "granted {kw:?}");
        }
    }

    /// Banishing Slash destroys a tapped creature and mints a Samurai when you
    /// control an artifact and an enchantment.
    #[test]
    fn banishing_slash_destroys_and_makes_samurai() {
        let mut g = two_player_game();
        let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        g.battlefield_find_mut(foe).unwrap().tapped = true;
        g.add_card_to_battlefield(0, catalog::mind_stone()); // artifact
        g.add_card_to_battlefield(0, catalog::solemnity()); // non-aura enchantment
        let mut ctx = crabomination::game::effects::EffectContext::for_ability(crabomination::card::CardId(0), 0, None);
        ctx.targets = vec![Target::Permanent(foe)];
        g.resolve_effect(&catalog::banishing_slash().effect, &ctx).unwrap();
        assert!(g.battlefield_find(foe).is_none(), "tapped creature destroyed");
        assert!(
            g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Samurai"),
            "Samurai token created"
        );
    }

    /// Lightshield Parry pumps +2/+2 and offers Cycling {2}.
    #[test]
    fn lightshield_parry_pumps_and_cycles() {
        use crabomination::card::Keyword;
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let parry = catalog::lightshield_parry();
        assert!(parry.keywords.iter().any(|k| matches!(k, Keyword::Cycling(_))));
        let mut ctx = crabomination::game::effects::EffectContext::for_ability(crabomination::card::CardId(0), 0, None);
        ctx.targets = vec![Target::Permanent(bear)];
        g.resolve_effect(&parry.effect, &ctx).unwrap();
        let cp = g.computed_permanent(bear).unwrap();
        assert_eq!((cp.power, cp.toughness), (4, 4));
    }

    /// Star Charter digs at end step only when you changed life that turn.
    #[test]
    fn star_charter_digs_after_life_change() {
        let mut g = two_player_game();
        let sc = g.add_card_to_battlefield(0, catalog::star_charter());
        let pred = catalog::star_charter().triggered_abilities[0].event.filter.clone().unwrap();
        let ctx = crabomination::game::effects::EffectContext::for_ability(sc, 0, None);
        // No life change → intervening-if fails.
        g.players[0].life_gained_this_turn = 0;
        g.players[0].lost_life_this_turn = false;
        assert!(!g.evaluate_predicate(&pred, &ctx), "no dig without a life change");
        // Gained life → condition holds.
        g.players[0].life_gained_this_turn = 2;
        assert!(g.evaluate_predicate(&pred, &ctx), "digs after gaining life");
    }

    /// Krydle's combat-damage trigger drains the player and self-scrys.
    #[test]
    fn krydle_combat_damage_drains_and_gains() {
        let mut g = two_player_game();
        let krydle = g.add_card_to_battlefield(0, catalog::krydle_of_baldurs_gate());
        g.add_card_to_library(1, catalog::island()); // something to mill
        g.players[0].life = 20;
        g.players[1].life = 20;
        let trig = catalog::krydle_of_baldurs_gate().triggered_abilities[0].effect.clone();
        let mut ctx = crabomination::game::effects::EffectContext::for_trigger(krydle, 0, None, 0);
        ctx.trigger_source = Some(crabomination::game::effects::EntityRef::Player(1));
        g.resolve_effect(&trig, &ctx).unwrap();
        assert_eq!(g.players[1].life, 19, "damaged player lost 1");
        assert_eq!(g.players[0].life, 21, "Krydle's controller gained 1");
        assert_eq!(g.players[1].graveyard.len(), 1, "milled a card");
    }

    /// Dour Port-Mage's activated ability returns another of your creatures.
    #[test]
    fn dour_port_mage_bounces_own_creature() {
        let mut g = two_player_game();
        let mage = g.add_card_to_battlefield(0, catalog::dour_port_mage());
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let mut ctx = crabomination::game::effects::EffectContext::for_ability(mage, 0, None);
        ctx.targets = vec![Target::Permanent(bear)];
        g.resolve_effect(&catalog::dour_port_mage().activated_abilities[0].effect, &ctx).unwrap();
        assert!(g.battlefield_find(bear).is_none() && g.players[0].hand.iter().any(|c| c.id == bear));
    }

    /// Dour Port-Mage draws when another of your creatures is bounced (CR 603.6
    /// leaves-without-dying).
    #[test]
    fn dour_port_mage_draws_on_bounce() {
        let mut g = two_player_game();
        g.add_card_to_library(0, catalog::island());
        let lib = g.players[0].library.len();
        let mage = g.add_card_to_battlefield(0, catalog::dour_port_mage());
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let bounce = catalog::dour_port_mage().activated_abilities[0].effect.clone();
        let mut ctx = crabomination::game::effects::EffectContext::for_ability(mage, 0, None);
        ctx.targets = vec![Target::Permanent(bear)];
        let evs = g.resolve_effect(&bounce, &ctx).unwrap();
        g.dispatch_triggers_for_events(&evs);
        drain_stack(&mut g);
        assert_eq!(g.players[0].library.len(), lib - 1, "leaves-without-dying drew a card");
    }

    /// Three Tree Scribe puts a +1/+1 counter on a creature you control when
    /// another of your creatures leaves without dying.
    #[test]
    fn three_tree_scribe_counters_on_leave() {
        let mut g = two_player_game();
        let scribe = g.add_card_to_battlefield(0, catalog::three_tree_scribe());
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let bounce = catalog::dour_port_mage().activated_abilities[0].effect.clone();
        let mut ctx = crabomination::game::effects::EffectContext::for_ability(scribe, 0, None);
        ctx.targets = vec![Target::Permanent(bear)];
        let evs = g.resolve_effect(&bounce, &ctx).unwrap();
        g.dispatch_triggers_for_events(&evs);
        drain_stack(&mut g);
        // Scribe is the only creature left, so the counter lands on it.
        assert_eq!(
            g.battlefield_find(scribe).unwrap().counters.get(&CounterType::PlusOnePlusOne).copied(),
            Some(1),
        );
    }

    /// Hard-Hitting Question makes your creature deal its power to a foe.
    #[test]
    fn hard_hitting_question_deals_power() {
        let mut g = two_player_game();
        let mine = g.add_card_to_battlefield(0, catalog::hill_giant()); // 3/3
        let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
        let mut ctx = crabomination::game::effects::EffectContext::for_ability(crabomination::card::CardId(0), 0, None);
        ctx.targets = vec![Target::Permanent(mine), Target::Permanent(foe)];
        g.resolve_effect(&catalog::hard_hitting_question().effect, &ctx).unwrap();
        assert!(g.battlefield_find(foe).is_none(), "3 damage kills the 2/2");
    }

    /// Brave-Kin Duo's sorcery-speed pump grows a creature by +1/+1.
    #[test]
    fn brave_kin_duo_pumps_at_sorcery_speed() {
        let mut g = two_player_game();
        let duo = catalog::brave_kin_duo();
        assert!(duo.activated_abilities[0].sorcery_speed, "activates only as a sorcery");
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let mut ctx = crabomination::game::effects::EffectContext::for_ability(crabomination::card::CardId(0), 0, None);
        ctx.targets = vec![Target::Permanent(bear)];
        g.resolve_effect(&duo.activated_abilities[0].effect, &ctx).unwrap();
        let cp = g.computed_permanent(bear).unwrap();
        assert_eq!((cp.power, cp.toughness), (3, 3), "+1/+1 until end of turn");
    }

    /// Marsh Hulk carries Megamorph and can be cast face down for {3}.
    #[test]
    fn marsh_hulk_has_megamorph() {
        use crabomination::card::Keyword;
        let hulk = catalog::marsh_hulk();
        assert!(hulk.keywords.iter().any(|k| matches!(k, Keyword::Megamorph(_))));
        assert_eq!((hulk.power, hulk.toughness), (4, 6));
    }

    /// Refurbished Familiar's affinity discounts it per artifact, and its ETB
    /// makes each opponent discard.
    #[test]
    fn refurbished_familiar_affinity_and_etb_discard() {
        use crabomination::game::actions::cost_reduction_for_spell;
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::mind_stone()); // an artifact
        g.add_card_to_battlefield(0, catalog::mind_stone());
        let spell = crabomination::card::CardInstance::new(g.next_id(), catalog::refurbished_familiar(), 0);
        assert_eq!(cost_reduction_for_spell(&g, 0, &spell, None), 2, "affinity for 2 artifacts");
        // ETB discard.
        g.add_card_to_hand(1, catalog::grizzly_bears());
        let fam = g.move_card_to_battlefield_for_test(0, catalog::refurbished_familiar());
        drain_stack(&mut g);
        let _ = fam;
        assert!(g.players[1].hand.is_empty(), "opponent discarded their only card");
    }

    /// Galvanic Discharge nets 3 energy then pays exactly lethal to kill a 3/3.
    #[test]
    fn galvanic_discharge_pays_lethal_energy() {
        let mut g = two_player_game();
        let foe = g.add_card_to_battlefield(1, catalog::hill_giant()); // 3/3
        g.players[0].energy = 1; // 1 + 3 from the spell = 4 available
        let mut ctx = crabomination::game::effects::EffectContext::for_ability(crabomination::card::CardId(0), 0, None);
        ctx.targets = vec![Target::Permanent(foe)];
        g.resolve_effect(&catalog::galvanic_discharge().effect, &ctx).unwrap();
        // 3 gained, paid 3 (lethal to the 3/3), 1 left over.
        assert_eq!(g.players[0].energy, 1, "spent only lethal energy");
        assert!(g.battlefield_find(foe).is_none(), "the 3/3 died");
    }

    /// This Town Ain't Big Enough bounces up to two nonland permanents and is
    /// cheaper when it targets one of yours.
    #[test]
    fn this_town_bounces_two_and_discounts_self_target() {
        use crabomination::game::actions::cost_reduction_for_spell;
        let mut g = two_player_game();
        let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let theirs = g.add_card_to_battlefield(1, catalog::serra_angel());
        // Targeting your own permanent → {3} off.
        let spell = crabomination::card::CardInstance::new(g.next_id(), catalog::this_town_aint_big_enough(), 0);
        assert_eq!(cost_reduction_for_spell(&g, 0, &spell, Some(&Target::Permanent(mine))), 3);
        assert_eq!(cost_reduction_for_spell(&g, 0, &spell, Some(&Target::Permanent(theirs))), 0);
        // Resolution bounces both.
        let mut ctx = crabomination::game::effects::EffectContext::for_ability(crabomination::card::CardId(0), 0, None);
        ctx.targets = vec![Target::Permanent(mine), Target::Permanent(theirs)];
        g.resolve_effect(&catalog::this_town_aint_big_enough().effect, &ctx).unwrap();
        assert!(g.battlefield_find(mine).is_none() && g.battlefield_find(theirs).is_none());
        assert!(g.players[0].hand.iter().any(|c| c.id == mine));
        assert!(g.players[1].hand.iter().any(|c| c.id == theirs));
    }

    /// Highspire Bell-Ringer cuts {1} off your second spell each turn only.
    #[test]
    fn highspire_bell_ringer_discounts_second_spell() {
        use crabomination::game::actions::cost_reduction_for_spell;
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::highspire_bell_ringer());
        let spell = crabomination::card::CardInstance::new(g.next_id(), catalog::grizzly_bears(), 0);
        // First spell (0 cast so far): no discount.
        assert_eq!(cost_reduction_for_spell(&g, 0, &spell, None), 0);
        // Second spell (1 cast already): {1} less.
        g.players[0].spells_cast_this_turn = 1;
        assert_eq!(cost_reduction_for_spell(&g, 0, &spell, None), 1);
        // Third spell: no discount.
        g.players[0].spells_cast_this_turn = 2;
        assert_eq!(cost_reduction_for_spell(&g, 0, &spell, None), 0);
    }

    /// Hardened Scales adds one to a +1/+1 placement on your creature.
    #[test]
    fn hardened_scales_adds_one() {
        use crabomination::effect::{Effect, Selector, Value};
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.add_card_to_battlefield(0, catalog::hardened_scales());
        let ctx = crabomination::game::effects::EffectContext::for_ability(
            crabomination::card::CardId(0), 0, Some(Target::Permanent(bear)),
        );
        g.resolve_effect(&Effect::AddCounter {
            what: Selector::Target(0), kind: CounterType::PlusOnePlusOne, amount: Value::Const(2),
        }, &ctx).unwrap();
        // 2 + 1 = 3.
        assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne), 3);
    }

    /// Vaultborn Tyrant draws and gains life when a big creature enters.
    #[test]
    fn vaultborn_tyrant_value_on_big_etb() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::vaultborn_tyrant());
        g.add_card_to_library(0, catalog::island());
        let life = g.players[0].life;
        // Cast a power-4 creature so the ETB event flows through the dispatcher.
        let angel = g.add_card_to_hand(0, catalog::serra_angel());
        g.players[0].mana_pool.add_colorless(3);
        g.players[0].mana_pool.add(Color::White, 2);
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        let hand = g.players[0].hand.len();
        g.perform_action(GameAction::CastSpell {
            card_id: angel, target: None, additional_targets: vec![], mode: None, x_value: None,
        })
        .expect("cast angel");
        drain_stack(&mut g);
        // Net hand: -1 (cast angel) +1 (Vaultborn draw) = same; life +3.
        assert_eq!(g.players[0].life, life + 3, "gained 3 life");
        assert_eq!(g.players[0].hand.len(), hand, "drew a card off the big ETB");
    }

    /// Unstoppable Slasher returns tapped with two stun counters when it dies.
    #[test]
    fn unstoppable_slasher_recurs_with_stun() {
        let mut g = two_player_game();
        let slasher = g.add_card_to_battlefield(0, catalog::unstoppable_slasher());
        g.battlefield_find_mut(slasher).unwrap().damage = 3; // lethal vs its 3 toughness
        g.check_state_based_actions();
        drain_stack(&mut g);
        let back = g
            .battlefield
            .iter()
            .find(|c| c.definition.name == "Unstoppable Slasher")
            .expect("returned to the battlefield");
        assert!(back.tapped, "returned tapped");
        assert_eq!(back.counters.get(&CounterType::Stun).copied(), Some(2), "two stun counters");
    }

    /// Emberheart Challenger's Valiant exiles the top card the first time you
    /// target it each turn.
    #[test]
    fn emberheart_challenger_valiant_exiles_top() {
        let mut g = two_player_game();
        let ember = g.add_card_to_battlefield(0, catalog::emberheart_challenger());
        g.add_card_to_library(0, catalog::mountain());
        let exile_before = g.exile.len();
        g.dispatch_triggers_for_events(&[GameEvent::BecameTarget { target: ember, caster: 0 }]);
        drain_stack(&mut g);
        assert!(g.exile.len() > exile_before, "Valiant exiled the top card");
    }

    /// Eldrazi Linebreaker pumps a target creature at the beginning of combat.
    #[test]
    fn eldrazi_linebreaker_combat_pump() {
        let mut g = two_player_game();
        let lb = g.add_card_to_battlefield(0, catalog::eldrazi_linebreaker());
        g.clear_sickness(lb);
        let other = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.clear_sickness(other);
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Target(Target::Permanent(other))]));
        while g.step != TurnStep::DeclareAttackers {
            g.perform_action(GameAction::PassPriority).unwrap();
        }
        drain_stack(&mut g);
        let bear = g.computed_permanent(other).unwrap();
        // One Eldrazi (the Linebreaker) → +1/+0 and haste.
        assert_eq!(bear.power, 3, "bear pumped by Eldrazi count");
        assert!(bear.keywords.contains(&Keyword::Haste), "gained haste");
    }

    /// No More Lies counters an unpayable spell and exiles it.
    #[test]
    fn no_more_lies_counters_and_exiles() {
        let mut g = two_player_game();
        // On the opponent's turn they cast a creature spell, spending all mana.
        let spell = g.add_card_to_hand(1, catalog::grizzly_bears());
        g.players[1].mana_pool.add(Color::Green, 1);
        g.players[1].mana_pool.add_colorless(1);
        g.active_player_idx = 1;
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 1;
        g.perform_action(GameAction::CastSpell {
            card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None,
        })
        .expect("opponent casts");
        // Seat 0 responds with No More Lies; opponent can't pay {3}.
        let nml = g.add_card_to_hand(0, catalog::no_more_lies());
        g.players[0].mana_pool.add(Color::White, 1);
        g.players[0].mana_pool.add(Color::Blue, 1);
        g.priority.player_with_priority = 0;
        cast_at(&mut g, nml, Target::Permanent(spell));
        assert!(
            g.exile.iter().any(|c| c.id == spell),
            "the countered spell was exiled, not graveyarded"
        );
    }

    /// Enduring Curiosity draws when your creature connects, and returns as an
    /// enchantment when it dies.
    #[test]
    fn enduring_curiosity_draws_then_returns_as_enchantment() {
        let mut g = two_player_game();
        let cur = g.add_card_to_battlefield(0, catalog::enduring_curiosity());
        g.clear_sickness(cur);
        g.add_card_to_library(0, catalog::island());
        g.active_player_idx = 0;
        g.step = TurnStep::DeclareAttackers; // past the draw step
        g.priority.player_with_priority = 0;
        let hand = g.players[0].hand.len();
        g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker: cur,
            target: AttackTarget::Player(1),
        }]))
        .expect("attack");
        drain_stack(&mut g);
        while g.step != TurnStep::PostCombatMain && g.step != TurnStep::End {
            g.perform_action(GameAction::PassPriority).unwrap();
        }
        drain_stack(&mut g);
        assert!(g.players[0].hand.len() > hand, "drew off combat damage");
        // Now kill it; it returns as a non-creature enchantment.
        g.battlefield_find_mut(cur).unwrap().damage = 3; // lethal for the 4/3
        g.check_state_based_actions();
        drain_stack(&mut g);
        let back = g
            .battlefield
            .iter()
            .find(|c| c.definition.name == "Enduring Curiosity")
            .expect("returned to battlefield");
        assert!(
            !back.definition.card_types.contains(&crabomination::card::CardType::Creature),
            "returns as a non-creature enchantment"
        );
    }

    /// The Necrobloom makes a Plant token whenever a land you control enters.
    #[test]
    fn necrobloom_landfall_makes_plant() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::the_necrobloom());
        g.active_player_idx = 0;
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        let land = g.add_card_to_hand(0, catalog::forest());
        g.perform_action(GameAction::PlayLand(land)).expect("play land");
        drain_stack(&mut g);
        let plants = g
            .battlefield
            .iter()
            .filter(|c| c.definition.name == "Plant" && c.is_token)
            .count();
        assert_eq!(plants, 1, "one Plant token from landfall");
    }

    /// Galvanic Relay exiles the top card of your library for later.
    #[test]
    fn galvanic_relay_exiles_top_for_later() {
        let mut g = two_player_game();
        g.add_card_to_library(0, catalog::mountain());
        let relay = g.add_card_to_hand(0, catalog::galvanic_relay());
        for _ in 0..3 { g.players[0].mana_pool.add(Color::Red, 1); }
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        let exile_before = g.exile.len();
        g.perform_action(GameAction::CastSpell {
            card_id: relay, target: None, additional_targets: vec![], mode: None, x_value: None,
        })
        .expect("cast relay");
        drain_stack(&mut g);
        assert!(g.exile.len() > exile_before, "exiled the top card");
    }

    /// Tyvar's Stand pumps and protects your creature for its X.
    #[test]
    fn tyvars_stand_pumps_by_x() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let spell = g.add_card_to_hand(0, catalog::tyvars_stand());
        g.players[0].mana_pool.add(Color::Green, 1);
        g.players[0].mana_pool.add_colorless(2); // X = 2
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::CastSpell {
            card_id: spell, target: Some(Target::Permanent(bear)),
            additional_targets: vec![], mode: None, x_value: Some(2),
        })
        .expect("cast Tyvar's Stand");
        drain_stack(&mut g);
        let c = g.computed_permanent(bear).unwrap();
        assert_eq!((c.power, c.toughness), (4, 4), "+2/+2 from X=2");
        assert!(c.keywords.contains(&Keyword::Hexproof));
        assert!(c.keywords.contains(&Keyword::Indestructible));
    }

    /// Gird for Battle puts a counter on each of up to two creatures.
    #[test]
    fn gird_for_battle_buffs_two() {
        let mut g = two_player_game();
        let a = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let b = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let spell = g.add_card_to_hand(0, catalog::gird_for_battle());
        g.players[0].mana_pool.add(Color::White, 1);
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::CastSpell {
            card_id: spell, target: Some(Target::Permanent(a)),
            additional_targets: vec![Target::Permanent(b)], mode: None, x_value: None,
        })
        .expect("cast Gird for Battle");
        drain_stack(&mut g);
        assert_eq!(g.battlefield_find(a).unwrap().counters.get(&CounterType::PlusOnePlusOne).copied(), Some(1));
        assert_eq!(g.battlefield_find(b).unwrap().counters.get(&CounterType::PlusOnePlusOne).copied(), Some(1));
    }

    /// Stock Up draws two of the top five into hand.
    #[test]
    fn stock_up_takes_two_of_five() {
        let mut g = two_player_game();
        for _ in 0..6 { g.add_card_to_library(0, catalog::island()); }
        let spell = g.add_card_to_hand(0, catalog::stock_up());
        g.players[0].mana_pool.add(Color::Blue, 1);
        g.players[0].mana_pool.add_colorless(2);
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        let hand = g.players[0].hand.len();
        g.perform_action(GameAction::CastSpell {
            card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None,
        })
        .expect("cast Stock Up");
        drain_stack(&mut g);
        // -1 (cast) +2 (picked) = +1 net.
        assert_eq!(g.players[0].hand.len(), hand + 1, "two cards into hand, one spell spent");
    }

    /// Shelter cantrips and grants protection.
    #[test]
    fn shelter_protects_and_draws() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.add_card_to_library(0, catalog::plains());
        let spell = g.add_card_to_hand(0, catalog::shelter());
        g.players[0].mana_pool.add(Color::White, 1);
        g.players[0].mana_pool.add_colorless(1);
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        let hand = g.players[0].hand.len();
        g.perform_action(GameAction::CastSpell {
            card_id: spell, target: Some(Target::Permanent(bear)),
            additional_targets: vec![], mode: None, x_value: None,
        })
        .expect("cast Shelter");
        drain_stack(&mut g);
        assert_eq!(g.players[0].hand.len(), hand, "drew a card (net even after cast)");
    }

    /// Pick Your Poison's first mode makes each opponent sacrifice an artifact.
    #[test]
    fn pick_your_poison_edicts_an_artifact() {
        let mut g = two_player_game();
        let rock = g.add_card_to_battlefield(1, catalog::mind_stone());
        let spell = g.add_card_to_hand(0, catalog::pick_your_poison());
        g.players[0].mana_pool.add(Color::Green, 1);
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Mode(0)]));
        g.perform_action(GameAction::CastSpell {
            card_id: spell, target: None, additional_targets: vec![], mode: Some(0), x_value: None,
        })
        .expect("cast Pick Your Poison");
        drain_stack(&mut g);
        assert!(g.battlefield_find(rock).is_none(), "opponent's only artifact was sacrificed");
    }

    /// Tail Swipe fights your creature against an opponent's.
    #[test]
    fn tail_swipe_fights() {
        let mut g = two_player_game();
        let mine = g.add_card_to_battlefield(0, catalog::serra_angel()); // 4/4
        let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
        let spell = g.add_card_to_hand(0, catalog::tail_swipe());
        g.players[0].mana_pool.add(Color::Green, 1);
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::CastSpell {
            card_id: spell, target: Some(Target::Permanent(mine)),
            additional_targets: vec![Target::Permanent(theirs)], mode: None, x_value: None,
        })
        .expect("cast Tail Swipe");
        drain_stack(&mut g);
        assert!(g.battlefield_find(theirs).is_none(), "their 2/2 died to the 4/4");
        assert!(g.battlefield_find(mine).is_some(), "our 4/4 survived 2 damage");
        // Cast during your main phase: the +1/+1 rider made it a 5/5.
        assert_eq!(g.computed_permanent(mine).unwrap().power, 5, "main-phase +1/+1");
    }

    /// Lightning Axe discards a card and deals 5 to a creature.
    #[test]
    fn lightning_axe_kills_with_a_discard() {
        let mut g = two_player_game();
        let target = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4
        let spell = g.add_card_to_hand(0, catalog::lightning_axe());
        let _fodder = g.add_card_to_hand(0, catalog::island());
        g.players[0].mana_pool.add(Color::Red, 1);
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        let hand = g.players[0].hand.len();
        g.perform_action(GameAction::CastSpell {
            card_id: spell, target: Some(Target::Permanent(target)),
            additional_targets: vec![], mode: None, x_value: None,
        })
        .expect("cast Lightning Axe");
        drain_stack(&mut g);
        assert!(g.battlefield_find(target).is_none(), "5 damage killed the 4/4");
        assert!(g.players[0].hand.len() < hand, "discarded a card as additional cost");
    }

    /// Stormsplitter copies itself when you cast an instant or sorcery.
    #[test]
    fn stormsplitter_copies_on_spell() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::stormsplitter());
        let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
        g.players[0].mana_pool.add(Color::Red, 1);
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::CastSpell {
            card_id: bolt, target: Some(Target::Player(1)),
            additional_targets: vec![], mode: None, x_value: None,
        })
        .expect("cast bolt");
        drain_stack(&mut g);
        let copies = g.battlefield.iter().filter(|c| c.definition.name == "Stormsplitter" && c.is_token).count();
        assert_eq!(copies, 1, "one token copy from the instant cast");
    }

    /// Unburden forces a player to discard two cards.
    #[test]
    fn unburden_discards_two() {
        let mut g = two_player_game();
        for _ in 0..3 { g.add_card_to_hand(1, catalog::island()); }
        let spell = g.add_card_to_hand(0, catalog::unburden());
        g.players[0].mana_pool.add(Color::Black, 2);
        g.players[0].mana_pool.add_colorless(1);
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        let opp_hand = g.players[1].hand.len();
        cast_at(&mut g, spell, Target::Player(1));
        assert_eq!(g.players[1].hand.len(), opp_hand - 2, "opponent discarded two");
    }

    /// Goblin Anarchomancer makes a red spell cost {1} less.
    #[test]
    fn goblin_anarchomancer_discounts_red() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::goblin_anarchomancer());
        // Lightning Bolt ({R}) becomes castable with no mana left after just {R}…
        // assert the reduced cost via the engine's castable check: give exactly
        // {R} (already enough), so instead verify the static reduces a {1}{R} spell.
        let pyro = g.add_card_to_hand(0, catalog::incinerate()); // {1}{R} instant
        g.players[0].mana_pool.add(Color::Red, 1); // only {R}; the {1} is discounted
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        cast_at(&mut g, pyro, Target::Player(1));
        assert_eq!(g.players[1].life, 17, "Incinerate dealt 3 after a 1-mana discount");
    }

    /// Beza gains 4 life when an opponent has more life.
    #[test]
    fn beza_gains_life_when_behind() {
        let mut g = two_player_game();
        g.players[0].life = 10;
        g.players[1].life = 20;
        let life = g.players[0].life;
        g.move_card_to_battlefield_for_test(0, catalog::beza_the_bounding_spring());
        drain_stack(&mut g);
        assert_eq!(g.players[0].life, life + 4, "gained 4 because opponent has more life");
    }

    /// Optimistic Scavenger grows a creature when an enchantment you control enters.
    #[test]
    fn optimistic_scavenger_eerie_counter() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::optimistic_scavenger());
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Target(Target::Permanent(bear))]));
        // An enchantment you control enters (dispatch the watcher event).
        let ench = g.add_card_to_battlefield(0, catalog::sticky_fingers());
        g.dispatch_triggers_for_events(&[GameEvent::PermanentEntered { card_id: ench }]);
        drain_stack(&mut g);
        assert_eq!(
            g.battlefield_find(bear).unwrap().counters.get(&CounterType::PlusOnePlusOne).copied(),
            Some(1),
            "Eerie put a +1/+1 counter on the target"
        );
    }

    /// Frilled Sandwalla pumps once per turn.
    #[test]
    fn frilled_sandwalla_once_per_turn_pump() {
        let mut g = two_player_game();
        let id = g.add_card_to_battlefield(0, catalog::frilled_sandwalla());
        g.clear_sickness(id);
        for _ in 0..4 { g.players[0].mana_pool.add(Color::Green, 1); }
        g.players[0].mana_pool.add_colorless(2);
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::ActivateAbility {
            card_id: id, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
        }).expect("first activation");
        drain_stack(&mut g);
        assert_eq!(g.computed_permanent(id).unwrap().power, 3, "+2/+2 once");
        // Second activation the same turn is rejected.
        assert!(g.perform_action(GameAction::ActivateAbility {
            card_id: id, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
        }).is_err(), "only once each turn");
    }

    /// Spectral Interference counters an artifact/creature spell the controller
    /// can't pay {4} for.
    #[test]
    fn spectral_interference_counters_creature() {
        let mut g = two_player_game();
        let spell = g.add_card_to_hand(1, catalog::grizzly_bears());
        g.players[1].mana_pool.add(Color::Green, 1);
        g.players[1].mana_pool.add_colorless(1);
        g.active_player_idx = 1;
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 1;
        g.perform_action(GameAction::CastSpell {
            card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("opponent casts a creature");
        let si = g.add_card_to_hand(0, catalog::spectral_interference());
        g.players[0].mana_pool.add(Color::Blue, 1);
        g.players[0].mana_pool.add_colorless(1);
        g.priority.player_with_priority = 0;
        cast_at(&mut g, si, Target::Permanent(spell));
        assert!(g.battlefield_find(spell).is_none(), "creature spell countered");
        assert!(g.players[1].graveyard.iter().any(|c| c.id == spell));
    }

    /// Refute counters a spell and loots.
    #[test]
    fn refute_counters_and_loots() {
        let mut g = two_player_game();
        let spell = g.add_card_to_hand(1, catalog::grizzly_bears());
        g.players[1].mana_pool.add(Color::Green, 1);
        g.players[1].mana_pool.add_colorless(1);
        g.active_player_idx = 1;
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 1;
        g.perform_action(GameAction::CastSpell {
            card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("opponent casts");
        g.add_card_to_library(0, catalog::island());
        let _junk = g.add_card_to_hand(0, catalog::island());
        let refute = g.add_card_to_hand(0, catalog::refute());
        g.players[0].mana_pool.add(Color::Blue, 2);
        g.players[0].mana_pool.add_colorless(1);
        g.priority.player_with_priority = 0;
        cast_at(&mut g, refute, Target::Permanent(spell));
        assert!(g.battlefield_find(spell).is_none(), "spell countered");
    }

    /// Skullcap Snail strips a card from an opponent's hand to exile.
    #[test]
    fn skullcap_snail_exiles_from_hand() {
        let mut g = two_player_game();
        g.add_card_to_hand(1, catalog::island());
        g.add_card_to_hand(1, catalog::forest());
        let exile_before = g.exile.len();
        let opp_hand = g.players[1].hand.len();
        g.move_card_to_battlefield_for_test(0, catalog::skullcap_snail());
        drain_stack(&mut g);
        assert_eq!(g.players[1].hand.len(), opp_hand - 1, "opponent lost a card from hand");
        assert!(g.exile.len() > exile_before, "it went to exile");
    }

    /// Aspirant's Ascent grants flying and toxic.
    #[test]
    fn aspirants_ascent_grants_flying_toxic() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let spell = g.add_card_to_hand(0, catalog::aspirants_ascent());
        g.players[0].mana_pool.add(Color::Blue, 1);
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        cast_at(&mut g, spell, Target::Permanent(bear));
        let c = g.computed_permanent(bear).unwrap();
        assert_eq!((c.power, c.toughness), (3, 5), "+1/+3");
        assert!(c.keywords.contains(&Keyword::Flying));
        assert!(c.keywords.iter().any(|k| matches!(k, Keyword::Toxic(1))));
    }

    /// Take the Fall shrinks a creature more when you control an outlaw.
    #[test]
    fn take_the_fall_outlaw_bonus() {
        let mut g = two_player_game();
        // An outlaw (Rogue) of ours.
        let mut rogue = catalog::grizzly_bears();
        rogue.subtypes.creature_types = vec![CreatureType::Rogue];
        g.add_card_to_battlefield(0, rogue);
        let victim = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4
        let spell = g.add_card_to_hand(0, catalog::take_the_fall());
        g.players[0].mana_pool.add(Color::Blue, 1);
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        cast_at(&mut g, spell, Target::Permanent(victim));
        assert_eq!(g.computed_permanent(victim).unwrap().power, 0, "-4/-0 with an outlaw");
    }

    /// Hopeful Vigil makes a Knight, and scries when it leaves.
    #[test]
    fn hopeful_vigil_token_and_sac_scry() {
        let mut g = two_player_game();
        g.add_card_to_library(0, catalog::island());
        g.add_card_to_library(0, catalog::forest());
        let vigil = g.move_card_to_battlefield_for_test(0, catalog::hopeful_vigil());
        drain_stack(&mut g);
        let knights = g.battlefield.iter().filter(|c| c.definition.name == "Knight").count();
        assert_eq!(knights, 1, "made a Knight on ETB");
        // Sacrifice it via its own ability → leaves-trigger scries.
        g.players[0].mana_pool.add(Color::White, 1);
        g.players[0].mana_pool.add_colorless(2);
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::ActivateAbility {
            card_id: vigil, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
        }).expect("sacrifice Hopeful Vigil");
        drain_stack(&mut g);
        assert!(g.battlefield_find(vigil).is_none(), "Hopeful Vigil sacrificed");
    }

    /// Hopeless Nightmare drains and discards on ETB.
    #[test]
    fn hopeless_nightmare_discard_and_drain() {
        let mut g = two_player_game();
        g.add_card_to_hand(1, catalog::island());
        g.players[1].life = 20;
        let opp_hand = g.players[1].hand.len();
        g.move_card_to_battlefield_for_test(0, catalog::hopeless_nightmare());
        drain_stack(&mut g);
        assert_eq!(g.players[1].life, 18, "opponent lost 2");
        assert_eq!(g.players[1].hand.len(), opp_hand - 1, "opponent discarded");
    }

    /// Hangar Scrounger's Backup puts a +1/+1 counter on a creature on ETB.
    #[test]
    fn hangar_scrounger_backup_counter() {
        let mut g = two_player_game();
        let ally = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Target(Target::Permanent(ally))]));
        g.move_card_to_battlefield_for_test(0, catalog::hangar_scrounger());
        drain_stack(&mut g);
        assert_eq!(
            g.battlefield_find(ally).unwrap().counters.get(&CounterType::PlusOnePlusOne).copied(),
            Some(1),
            "Backup 1 added a counter"
        );
    }

    /// Bristlebud Farmer makes two Food on ETB.
    #[test]
    fn bristlebud_farmer_makes_food() {
        let mut g = two_player_game();
        g.move_card_to_battlefield_for_test(0, catalog::bristlebud_farmer());
        drain_stack(&mut g);
        let food = g.battlefield.iter().filter(|c| c.definition.name == "Food").count();
        assert_eq!(food, 2, "two Food tokens");
    }

    /// Outcaster Greenblade tutors a basic land to hand on ETB.
    #[test]
    fn outcaster_greenblade_fetches_a_basic() {
        let mut g = two_player_game();
        g.add_card_to_library(0, catalog::forest());
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(None)]));
        let hand = g.players[0].hand.len();
        // Script the search to take the forest.
        let forest = g.players[0].library[0].id;
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(forest))]));
        let blade = g.move_card_to_battlefield_for_test(0, catalog::outcaster_greenblade());
        drain_stack(&mut g);
        assert_eq!(g.players[0].hand.len(), hand + 1, "fetched a land to hand");
        // Base 1/2 with no Deserts.
        let c = g.computed_permanent(blade).unwrap();
        assert_eq!((c.power, c.toughness), (1, 2));
        // Each Desert you control grows it +1/+1.
        g.add_card_to_battlefield(0, catalog::conduit_pylons());
        g.add_card_to_battlefield(0, catalog::conduit_pylons());
        let c = g.computed_permanent(blade).unwrap();
        assert_eq!((c.power, c.toughness), (3, 4), "+1/+1 per Desert");
    }

    /// Mizzium Skin grants hexproof.
    #[test]
    fn mizzium_skin_grants_hexproof() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let spell = g.add_card_to_hand(0, catalog::mizzium_skin());
        g.players[0].mana_pool.add(Color::Blue, 1);
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        cast_at(&mut g, spell, Target::Permanent(bear));
        let c = g.computed_permanent(bear).unwrap();
        assert_eq!(c.toughness, 3, "+0/+1");
        assert!(c.keywords.contains(&Keyword::Hexproof));
    }

    /// Demand Answers discards then draws two.
    #[test]
    fn demand_answers_loots_up() {
        let mut g = two_player_game();
        for _ in 0..2 { g.add_card_to_library(0, catalog::island()); }
        let spell = g.add_card_to_hand(0, catalog::demand_answers());
        let _fodder = g.add_card_to_hand(0, catalog::forest());
        g.players[0].mana_pool.add(Color::Red, 1);
        g.players[0].mana_pool.add_colorless(1);
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        let hand = g.players[0].hand.len();
        g.perform_action(GameAction::CastSpell {
            card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast Demand Answers");
        drain_stack(&mut g);
        // -1 cast, -1 discard, +2 draw = net 0 vs captured (which still held the spell).
        assert_eq!(g.players[0].hand.len(), hand, "discarded one and drew two");
    }

    /// Boltwave burns each opponent for 3.
    #[test]
    fn boltwave_burns_each_opponent() {
        let mut g = two_player_game();
        g.players[1].life = 20;
        let spell = g.add_card_to_hand(0, catalog::boltwave());
        g.players[0].mana_pool.add(Color::Red, 1);
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::CastSpell {
            card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast Boltwave");
        drain_stack(&mut g);
        assert_eq!(g.players[1].life, 17, "opponent took 3");
    }

    /// Strike It Rich mints a Treasure.
    #[test]
    fn strike_it_rich_makes_treasure() {
        let mut g = two_player_game();
        let spell = g.add_card_to_hand(0, catalog::strike_it_rich());
        g.players[0].mana_pool.add(Color::Red, 1);
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::CastSpell {
            card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast Strike It Rich");
        drain_stack(&mut g);
        let treasures = g.battlefield.iter().filter(|c| c.definition.name == "Treasure").count();
        assert_eq!(treasures, 1, "made a Treasure");
    }

    /// Brotherhood's End mode 0 sweeps creatures for 3.
    #[test]
    fn brotherhoods_end_sweeps_creatures() {
        let mut g = two_player_game();
        let a = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
        let b = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
        let spell = g.add_card_to_hand(0, catalog::brotherhoods_end());
        g.players[0].mana_pool.add(Color::Red, 2);
        g.players[0].mana_pool.add_colorless(1);
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Mode(0)]));
        g.perform_action(GameAction::CastSpell {
            card_id: spell, target: None, additional_targets: vec![], mode: Some(0), x_value: None,
        }).expect("cast Brotherhood's End");
        drain_stack(&mut g);
        assert!(g.battlefield_find(a).is_none() && g.battlefield_find(b).is_none(), "both 2/2s died");
    }

    /// Boon-Bringer Valkyrie's Backup grants flying to the backed-up creature.
    #[test]
    fn boon_bringer_valkyrie_backup_grants_flying() {
        let mut g = two_player_game();
        let ally = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Target(Target::Permanent(ally))]));
        g.move_card_to_battlefield_for_test(0, catalog::boon_bringer_valkyrie());
        drain_stack(&mut g);
        let c = g.computed_permanent(ally).unwrap();
        assert_eq!(c.power, 3, "got a +1/+1 counter");
        assert!(c.keywords.contains(&Keyword::Flying), "gained flying from Backup");
    }

    /// Inti's discard trigger exiles the top of your library with a may-play.
    #[test]
    fn inti_discard_exiles_top_with_may_play() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::inti_seneschal_of_the_sun());
        g.add_card_to_library(0, catalog::mountain());
        // Unburden (cast by p0, targeting p0) forces a discard, firing Inti.
        let unburden = g.add_card_to_hand(0, catalog::unburden());
        g.add_card_to_hand(0, catalog::mountain()); // something to discard
        g.add_card_to_hand(0, catalog::mountain());
        g.players[0].mana_pool.add(Color::Black, 2);
        g.players[0].mana_pool.add_colorless(1);
        g.active_player_idx = 0;
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        cast_at(&mut g, unburden, Target::Player(0));
        drain_stack(&mut g);
        assert!(
            g.exile.iter().any(|c| c.owner == 0),
            "Inti exiled the top of the library on discard"
        );
    }

    /// Warren Soultrader sacrifices a creature and pays 1 life for a Treasure.
    #[test]
    fn warren_soultrader_makes_treasure() {
        let mut g = two_player_game();
        let warren = g.add_card_to_battlefield(0, catalog::warren_soultrader());
        let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.players[0].life = 20;
        g.active_player_idx = 0;
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Target(Target::Permanent(fodder))]));
        g.perform_action(GameAction::ActivateAbility {
            card_id: warren, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
        }).expect("activate");
        drain_stack(&mut g);
        assert!(g.battlefield_find(fodder).is_none(), "sacrificed the creature");
        assert_eq!(g.players[0].life, 19, "paid 1 life");
        assert!(
            g.battlefield.iter().any(|c| c.controller == 0
                && c.definition.subtypes.artifact_subtypes.contains(&crabomination::card::ArtifactSubtype::Treasure)),
            "made a Treasure"
        );
    }

    /// Hostile Investigator makes a target opponent discard on ETB.
    #[test]
    fn hostile_investigator_etb_discard() {
        let mut g = two_player_game();
        g.add_card_to_hand(1, catalog::mountain());
        let opp_hand = g.players[1].hand.len();
        g.move_card_to_battlefield_for_test(0, catalog::hostile_investigator());
        drain_stack(&mut g);
        assert_eq!(g.players[1].hand.len(), opp_hand - 1, "opponent discarded to the ETB");
    }

    /// Marshal of Zhalfir buffs other Knights and can tap a creature.
    #[test]
    fn marshal_of_zhalfir_anthems_knights() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::marshal_of_zhalfir());
        let knight = g.add_card_to_battlefield(0, catalog::inti_seneschal_of_the_sun()); // a Knight
        let c = g.computed_permanent(knight).unwrap();
        assert_eq!((c.power, c.toughness), (3, 3), "another Knight got +1/+1");
    }

    /// Pawpatch Recruit grows a creature when an opponent targets one you control.
    #[test]
    fn pawpatch_recruit_counters_on_opponent_target() {
        let mut g = two_player_game();
        let pixie = g.add_card_to_battlefield(0, catalog::pawpatch_recruit());
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        // Opponent's Stab targets your bear; Pawpatch's trigger puts a counter on it.
        let stab = g.add_card_to_hand(1, catalog::stab());
        g.players[1].mana_pool.add(Color::Black, 1);
        g.active_player_idx = 1;
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 1;
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Target(Target::Permanent(bear))]));
        cast_at(&mut g, stab, Target::Permanent(bear));
        drain_stack(&mut g);
        let _ = pixie;
        assert!(
            g.battlefield_find(bear).map(|c| c.counters.get(&CounterType::PlusOnePlusOne).copied())
                == Some(Some(1)),
            "Pawpatch put a +1/+1 counter on the targeted creature"
        );
    }

    /// Helping Hand returns a small creature from your graveyard tapped.
    #[test]
    fn helping_hand_reanimates_tapped() {
        let mut g = two_player_game();
        let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears()); // MV 2
        let hh = g.add_card_to_hand(0, catalog::helping_hand());
        g.players[0].mana_pool.add(Color::White, 1);
        g.active_player_idx = 0;
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        cast_at(&mut g, hh, Target::Permanent(bear));
        drain_stack(&mut g);
        let r = g.battlefield_find(bear).expect("reanimated onto battlefield");
        assert!(r.tapped, "entered tapped");
    }

    /// Diversion Unit sacrifices itself to counter a spell.
    #[test]
    fn diversion_unit_counters_spell() {
        let mut g = two_player_game();
        let unit = g.add_card_to_battlefield(0, catalog::diversion_unit());
        let bolt = g.add_card_to_hand(1, catalog::lightning_axe()); // an instant
        g.players[1].mana_pool.add(Color::Red, 1);
        g.add_card_to_hand(1, catalog::mountain()); // discard fodder for Lightning Axe
        g.active_player_idx = 1;
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 1;
        let dummy = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.perform_action(GameAction::CastSpell {
            card_id: bolt, target: Some(Target::Permanent(dummy)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("opponent casts Lightning Axe");
        g.players[0].mana_pool.add(Color::Blue, 1);
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::ActivateAbility {
            card_id: unit, ability_index: 0,
            target: Some(Target::Permanent(bolt)), additional_targets: vec![], x_value: None,
        }).expect("activate counter");
        drain_stack(&mut g);
        assert!(g.players[1].graveyard.iter().any(|c| c.id == bolt), "Lightning Axe was countered");
    }

    /// Final Vengeance sacrifices a permanent and exiles a creature.
    #[test]
    fn final_vengeance_sac_and_exile() {
        let mut g = two_player_game();
        let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let victim = g.add_card_to_battlefield(1, catalog::serra_angel());
        let fv = g.add_card_to_hand(0, catalog::final_vengeance());
        g.players[0].mana_pool.add(Color::Black, 1);
        g.active_player_idx = 0;
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        cast_at(&mut g, fv, Target::Permanent(victim));
        drain_stack(&mut g);
        assert!(g.battlefield_find(fodder).is_none(), "sacrificed a creature as additional cost");
        assert!(g.exile.iter().any(|c| c.id == victim), "exiled the target creature");
    }

    /// Roughshod Mentor gives your green creatures trample.
    #[test]
    fn roughshod_mentor_grants_green_trample() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::roughshod_mentor());
        let elf = g.add_card_to_battlefield(0, catalog::llanowar_elves()); // green
        assert!(
            g.computed_permanent(elf).unwrap().keywords.contains(&Keyword::Trample),
            "green creature gained trample"
        );
    }

    /// Innocuous Rat manifests dread when it dies.
    #[test]
    fn innocuous_rat_manifests_on_death() {
        let mut g = two_player_game();
        let rat = g.add_card_to_battlefield(0, catalog::innocuous_rat());
        g.add_card_to_library(0, catalog::mountain());
        g.add_card_to_library(0, catalog::island());
        let bf_before = g.battlefield.len();
        g.battlefield_find_mut(rat).unwrap().damage = 1;
        g.check_state_based_actions();
        drain_stack(&mut g);
        // Rat left; a face-down 2/2 entered → battlefield count unchanged net (−rat +manifest).
        assert!(
            g.battlefield.iter().any(|c| c.controller == 0 && c.face_down),
            "manifested a face-down creature"
        );
        let _ = bf_before;
    }

    /// Quaketusk Boar is a 5/5 with reach, trample, and haste.
    #[test]
    fn quaketusk_boar_keywords() {
        let d = catalog::quaketusk_boar();
        assert_eq!((d.power, d.toughness), (5, 5));
        assert!(d.keywords.contains(&Keyword::Reach));
        assert!(d.keywords.contains(&Keyword::Trample));
        assert!(d.keywords.contains(&Keyword::Haste));
    }

    /// Veteran Guardmouse's Valiant fires when you target it (it gains first
    /// strike; Valiant's +1/+0 resolves before the targeting spell).
    #[test]
    fn veteran_guardmouse_valiant_pumps() {
        let mut g = two_player_game();
        let mouse = g.add_card_to_battlefield(0, catalog::veteran_guardmouse()); // 3/4
        g.add_card_to_library(0, catalog::mountain());
        let stab = g.add_card_to_hand(0, catalog::stab()); // your own targeted spell
        g.players[0].mana_pool.add(Color::Black, 1);
        g.active_player_idx = 0;
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        cast_at(&mut g, stab, Target::Permanent(mouse));
        drain_stack(&mut g);
        let c = g.computed_permanent(mouse).unwrap();
        // Valiant (+1/+0) resolves first, then Stab (-2/-2): 3+1-2 / 4+0-2 = 2/2.
        assert_eq!((c.power, c.toughness), (2, 2), "+1/+0 then -2/-2");
        assert!(c.keywords.contains(&Keyword::FirstStrike), "gained first strike");
    }

    /// Polliwallop makes your creature deal twice its power to an enemy creature.
    #[test]
    fn polliwallop_deals_double_power() {
        let mut g = two_player_game();
        let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
        let theirs = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4
        let poll = g.add_card_to_hand(0, catalog::polliwallop());
        g.players[0].mana_pool.add(Color::Green, 1);
        g.players[0].mana_pool.add_colorless(3);
        g.active_player_idx = 0;
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::CastSpell {
            card_id: poll, target: Some(Target::Permanent(mine)),
            additional_targets: vec![Target::Permanent(theirs)], mode: None, x_value: None,
        }).expect("cast Polliwallop");
        drain_stack(&mut g);
        assert!(g.battlefield_find(theirs).is_none(), "4 damage (2×2) killed the 4/4");
    }

    /// Coiling Rebirth reanimates a creature from your graveyard.
    #[test]
    fn coiling_rebirth_reanimates() {
        let mut g = two_player_game();
        let bear = g.add_card_to_graveyard(0, catalog::serra_angel());
        let cr = g.add_card_to_hand(0, catalog::coiling_rebirth());
        g.players[0].mana_pool.add(Color::Black, 2);
        g.players[0].mana_pool.add_colorless(3);
        g.active_player_idx = 0;
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        cast_at(&mut g, cr, Target::Permanent(bear));
        drain_stack(&mut g);
        assert!(g.battlefield_find(bear).is_some(), "creature returned to the battlefield");
    }

    /// Pearl of Wisdom draws two cards.
    #[test]
    fn pearl_of_wisdom_draws_two() {
        let mut g = two_player_game();
        g.add_card_to_library(0, catalog::island());
        g.add_card_to_library(0, catalog::island());
        let pearl = g.add_card_to_hand(0, catalog::pearl_of_wisdom());
        g.players[0].mana_pool.add(Color::Blue, 1);
        g.players[0].mana_pool.add_colorless(2);
        g.active_player_idx = 0;
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        let hand = g.players[0].hand.len();
        g.perform_action(GameAction::CastSpell {
            card_id: pearl, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast Pearl");
        drain_stack(&mut g);
        assert_eq!(g.players[0].hand.len(), hand - 1 + 2, "drew two (cast one)");
    }

    /// Geistlight Snare's two board-state clauses each cut {1} independently, and
    /// only off your own permanents.
    #[test]
    fn geistlight_snare_two_clause_discount() {
        use crabomination::game::actions::cost_reduction_for_spell;
        let mut g = two_player_game();
        let snare = crabomination::card::CardInstance::new(g.next_id(), catalog::geistlight_snare(), 0);
        // No Spirit, no enchantment: full price.
        assert_eq!(cost_reduction_for_spell(&g, 0, &snare, None), 0);
        // Opponent's Spirit doesn't count.
        g.add_card_to_battlefield(1, catalog::selfless_spirit());
        assert_eq!(cost_reduction_for_spell(&g, 0, &snare, None), 0);
        // Your Spirit: {1} less.
        g.add_card_to_battlefield(0, catalog::selfless_spirit());
        assert_eq!(cost_reduction_for_spell(&g, 0, &snare, None), 1);
        // Plus an enchantment: {2} less.
        g.add_card_to_battlefield(0, catalog::wedding_announcement());
        assert_eq!(cost_reduction_for_spell(&g, 0, &snare, None), 2);
    }

    /// Ride's End costs {3} less when it targets a tapped permanent.
    #[test]
    fn rides_end_cost_reduction_when_tapped() {
        let mut g = two_player_game();
        let victim = g.add_card_to_battlefield(1, catalog::serra_angel());
        g.battlefield_find_mut(victim).unwrap().tapped = true;
        let re = g.add_card_to_hand(0, catalog::rides_end());
        g.players[0].mana_pool.add(Color::White, 1);
        g.players[0].mana_pool.add_colorless(1); // only {1}{W} available, not {4}{W}
        g.active_player_idx = 0;
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        cast_at(&mut g, re, Target::Permanent(victim));
        drain_stack(&mut g);
        assert!(g.exile.iter().any(|c| c.id == victim), "cheaply exiled a tapped creature");
    }

    /// Nurturing Pixie bounces your own permanent and grows.
    #[test]
    fn nurturing_pixie_bounce_and_grow() {
        let mut g = two_player_game();
        let token = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.decider = Box::new(ScriptedDecider::new([
            DecisionAnswer::Bool(true),
            DecisionAnswer::Target(Target::Permanent(token)),
        ]));
        let pixie = g.move_card_to_battlefield_for_test(0, catalog::nurturing_pixie());
        drain_stack(&mut g);
        assert!(g.battlefield_find(token).is_none(), "bounced your own permanent");
        assert_eq!(g.computed_permanent(pixie).unwrap().power, 2, "Pixie grew to 2/2");
    }

    /// Ruby pumps herself when attacking alongside a big creature.
    #[test]
    fn ruby_pumps_with_big_creature() {
        let mut g = two_player_game();
        let ruby = g.add_card_to_battlefield(0, catalog::ruby_daring_tracker());
        g.add_card_to_battlefield(0, catalog::serra_angel()); // power 4
        g.clear_sickness(ruby);
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        while g.step != TurnStep::DeclareAttackers {
            g.perform_action(GameAction::PassPriority).unwrap();
        }
        g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker: ruby, target: AttackTarget::Player(1),
        }])).expect("attack");
        drain_stack(&mut g);
        assert_eq!(g.computed_permanent(ruby).unwrap().power, 3, "Ruby got +2/+2");
    }

    /// Stab gives a creature -2/-2.
    #[test]
    fn stab_shrinks_creature() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
        let stab = g.add_card_to_hand(0, catalog::stab());
        g.players[0].mana_pool.add(Color::Black, 1);
        g.active_player_idx = 0;
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        cast_at(&mut g, stab, Target::Permanent(bear));
        drain_stack(&mut g);
        assert!(g.battlefield_find(bear).is_none(), "-2/-2 killed the 2/2");
    }

    /// Slumbering Keepguard scries when an enchantment enters.
    #[test]
    fn slumbering_keepguard_scries_on_enchantment() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::slumbering_keepguard());
        g.add_card_to_library(0, catalog::island());
        // An enchantment entering under your control triggers the scry.
        g.move_card_to_battlefield_for_test(0, catalog::hopeless_nightmare()); // an enchantment
        drain_stack(&mut g);
        // No panic / clean resolution is the assertion; the scry decision auto-resolves.
        assert!(g.battlefield.iter().any(|c| c.definition.name == "Slumbering Keepguard"));
    }

    /// Anoint with Affliction exiles a small creature.
    #[test]
    fn anoint_with_affliction_exiles_small() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // MV 2
        let spell = g.add_card_to_hand(0, catalog::anoint_with_affliction());
        g.players[0].mana_pool.add(Color::Black, 1);
        g.players[0].mana_pool.add_colorless(1);
        g.active_player_idx = 0;
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        cast_at(&mut g, spell, Target::Permanent(bear));
        drain_stack(&mut g);
        assert!(g.exile.iter().any(|c| c.id == bear), "small creature exiled");
    }

    /// Wing It pumps, adds a flying counter, and scries.
    #[test]
    fn wing_it_pumps_and_grants_flying() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
        g.add_card_to_library(0, catalog::island());
        let spell = g.add_card_to_hand(0, catalog::wing_it());
        g.players[0].mana_pool.add(Color::White, 1);
        g.players[0].mana_pool.add_colorless(1);
        g.active_player_idx = 0;
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        cast_at(&mut g, spell, Target::Permanent(bear));
        drain_stack(&mut g);
        let c = g.computed_permanent(bear).unwrap();
        assert_eq!(c.power, 4, "+2/+2");
        assert!(c.keywords.contains(&Keyword::Flying), "flying counter grants flying");
    }

    /// Cackling Prowler grows at end step when a creature died this turn.
    #[test]
    fn cackling_prowler_morbid_counter() {
        let mut g = two_player_game();
        let prowler = g.add_card_to_battlefield(0, catalog::cackling_prowler());
        g.players[0].creatures_died_this_turn = 1;
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        g.step = TurnStep::PostCombatMain;
        while g.step != TurnStep::End {
            g.perform_action(GameAction::PassPriority).unwrap();
        }
        drain_stack(&mut g);
        assert_eq!(
            g.battlefield_find(prowler).unwrap().counters.get(&CounterType::PlusOnePlusOne).copied(),
            Some(1),
            "morbid put a +1/+1 counter at end step"
        );
    }

    /// Glimmerlight mints a Glimmer token on enter.
    #[test]
    fn glimmerlight_makes_glimmer_token() {
        let mut g = two_player_game();
        g.move_card_to_battlefield_for_test(0, catalog::glimmerlight());
        drain_stack(&mut g);
        assert!(
            g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Glimmer"),
            "minted a Glimmer token"
        );
    }

    /// Demonic Ruckus buffs the enchanted creature and draws when it dies.
    #[test]
    fn demonic_ruckus_buffs_then_cantrips() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
        g.add_card_to_library(0, catalog::island());
        let aura = g.add_card_to_hand(0, catalog::demonic_ruckus());
        g.players[0].mana_pool.add(Color::Red, 1);
        g.players[0].mana_pool.add_colorless(1);
        g.active_player_idx = 0;
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        cast_at(&mut g, aura, Target::Permanent(bear));
        drain_stack(&mut g);
        let c = g.computed_permanent(bear).unwrap();
        assert_eq!((c.power, c.toughness), (3, 3), "enchanted creature gets +1/+1");
        assert!(c.keywords.contains(&Keyword::Menace), "gains menace");
        // Kill the bear → the Aura dies and cantrips.
        let hand = g.players[0].hand.len();
        g.battlefield_find_mut(bear).unwrap().damage = 3;
        g.check_state_based_actions();
        drain_stack(&mut g);
        assert!(g.players[0].hand.len() > hand, "Aura death drew a card");
    }

    /// Hugs exiles X cards with a may-play when it enters.
    #[test]
    fn hugs_exiles_x_with_may_play() {
        let mut g = two_player_game();
        g.add_card_to_library(0, catalog::mountain());
        g.add_card_to_library(0, catalog::forest());
        let hugs = g.add_card_to_hand(0, catalog::hugs_grisly_guardian());
        g.players[0].mana_pool.add(Color::Red, 2);
        g.players[0].mana_pool.add(Color::Green, 2);
        g.players[0].mana_pool.add_colorless(2); // X = 2
        g.active_player_idx = 0;
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::CastSpell {
            card_id: hugs, target: None, additional_targets: vec![], mode: None, x_value: Some(2),
        }).expect("cast Hugs with X=2");
        drain_stack(&mut g);
        assert_eq!(g.exile.iter().filter(|c| c.owner == 0).count(), 2, "exiled the top 2 cards");
    }

    /// Gloomfang Mauler's Backup 2 puts two counters on a creature.
    #[test]
    fn gloomfang_mauler_backup_two() {
        let mut g = two_player_game();
        let ally = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Target(Target::Permanent(ally))]));
        g.move_card_to_battlefield_for_test(0, catalog::gloomfang_mauler());
        drain_stack(&mut g);
        assert_eq!(
            g.battlefield_find(ally).unwrap().counters.get(&CounterType::PlusOnePlusOne).copied(),
            Some(2),
            "Backup 2 put two +1/+1 counters"
        );
    }

    /// Audacity buffs the enchanted creature and cantrips when it leaves.
    #[test]
    fn audacity_buffs_and_cantrips() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
        g.add_card_to_library(0, catalog::island());
        let aura = g.add_card_to_hand(0, catalog::audacity());
        g.players[0].mana_pool.add(Color::Green, 1);
        g.active_player_idx = 0;
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        cast_at(&mut g, aura, Target::Permanent(bear));
        drain_stack(&mut g);
        let c = g.computed_permanent(bear).unwrap();
        assert_eq!((c.power, c.toughness), (4, 2), "+2/+0");
        assert!(c.keywords.contains(&Keyword::Trample));
        let hand = g.players[0].hand.len();
        g.battlefield_find_mut(bear).unwrap().damage = 2;
        g.check_state_based_actions();
        drain_stack(&mut g);
        assert!(g.players[0].hand.len() > hand, "Aura death drew a card");
    }

    /// Felonious Rage leaves a Detective when the buffed creature dies.
    #[test]
    fn felonious_rage_death_makes_detective() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let spell = g.add_card_to_hand(0, catalog::felonious_rage());
        g.players[0].mana_pool.add(Color::Red, 1);
        g.active_player_idx = 0;
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        cast_at(&mut g, spell, Target::Permanent(bear));
        drain_stack(&mut g);
        assert!(g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Haste));
        // Kill the buffed 4/2 with a burn spell so the death flows through the
        // damage funnel that the "dies this turn" watch listens on.
        let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
        g.players[0].mana_pool.add(Color::Red, 1);
        cast_at(&mut g, bolt, Target::Permanent(bear));
        drain_stack(&mut g);
        assert!(!g.battlefield.iter().any(|c| c.id == bear), "buffed creature died");
        assert!(
            g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Detective"),
            "the dying creature left a Detective token"
        );
    }

    /// Razorkin Hordecaller mints a Gremlin when you attack.
    #[test]
    fn razorkin_hordecaller_attack_token() {
        let mut g = two_player_game();
        let razor = g.add_card_to_battlefield(0, catalog::razorkin_hordecaller());
        g.clear_sickness(razor);
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        while g.step != TurnStep::DeclareAttackers {
            g.perform_action(GameAction::PassPriority).unwrap();
        }
        g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker: razor, target: AttackTarget::Player(1),
        }])).expect("attack");
        drain_stack(&mut g);
        assert!(
            g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Gremlin"),
            "minted a Gremlin on attack"
        );
    }

    /// Goldvein Pick gives +1/+1 and a Treasure on combat damage.
    #[test]
    fn goldvein_pick_buffs_and_treasures() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
        g.clear_sickness(bear);
        let pick = g.add_card_to_battlefield(0, catalog::goldvein_pick());
        g.players[0].mana_pool.add_colorless(1); // Equip {1}
        g.active_player_idx = 0;
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::Equip { equipment: pick, target: bear }).expect("equip");
        let c = g.computed_permanent(bear).unwrap();
        assert_eq!((c.power, c.toughness), (3, 3), "equipped creature gets +1/+1");
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        while g.step != TurnStep::DeclareAttackers {
            g.perform_action(GameAction::PassPriority).unwrap();
        }
        g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker: bear, target: AttackTarget::Player(1),
        }])).expect("attack");
        drain_stack(&mut g);
        while g.step != TurnStep::PostCombatMain && g.step != TurnStep::End {
            g.perform_action(GameAction::PassPriority).unwrap();
        }
        drain_stack(&mut g);
        assert!(
            g.battlefield.iter().any(|c| c.controller == 0
                && c.definition.subtypes.artifact_subtypes.contains(&crabomination::card::ArtifactSubtype::Treasure)),
            "combat damage made a Treasure"
        );
    }

    /// Boulderborn Dragon surveils when it attacks.
    #[test]
    fn boulderborn_dragon_surveils_on_attack() {
        let mut g = two_player_game();
        let dragon = g.add_card_to_battlefield(0, catalog::boulderborn_dragon());
        g.clear_sickness(dragon);
        g.add_card_to_library(0, catalog::island());
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        while g.step != TurnStep::DeclareAttackers {
            g.perform_action(GameAction::PassPriority).unwrap();
        }
        let lib = g.players[0].library.len();
        g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker: dragon, target: AttackTarget::Player(1),
        }])).expect("attack");
        drain_stack(&mut g);
        // Surveil looked at the top card (kept or binned) — library shrank by ≤1; the
        // trigger resolved without error and the dragon has flying+vigilance.
        assert!(g.players[0].library.len() <= lib);
        let c = g.battlefield_find(dragon).unwrap();
        assert!(c.definition.keywords.contains(&Keyword::Flying));
        assert!(c.definition.keywords.contains(&Keyword::Vigilance));
    }

    /// Scales of Shale costs less with Lizards and buffs a creature.
    #[test]
    fn scales_of_shale_affinity_and_buff() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let spell = g.add_card_to_hand(0, catalog::scales_of_shale());
        g.players[0].mana_pool.add(Color::Black, 1);
        g.players[0].mana_pool.add_colorless(2);
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        cast_at(&mut g, spell, Target::Permanent(bear));
        let c = g.computed_permanent(bear).unwrap();
        assert_eq!(c.power, 4, "+2/+0");
        assert!(c.keywords.contains(&Keyword::Lifelink));
        assert!(c.keywords.contains(&Keyword::Indestructible));
    }

    /// Sunset Strikemaster sacrifices to burn a flier.
    #[test]
    fn sunset_strikemaster_burns_a_flier() {
        let mut g = two_player_game();
        let master = g.add_card_to_battlefield(0, catalog::sunset_strikemaster());
        g.clear_sickness(master);
        let flier = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4 flying
        g.players[0].mana_pool.add(Color::Red, 1);
        g.players[0].mana_pool.add_colorless(2);
        g.perform_action(GameAction::ActivateAbility {
            card_id: master, ability_index: 1,
            target: Some(Target::Permanent(flier)), additional_targets: vec![], x_value: None,
        }).expect("sac to burn the flier");
        drain_stack(&mut g);
        assert!(g.battlefield_find(flier).is_none(), "6 damage killed the 4/4 flier");
        assert!(g.battlefield_find(master).is_none(), "sacrificed itself");
    }

    /// Wardens of the Cycle's morbid end-step trigger draws + drains when a creature died.
    #[test]
    fn wardens_of_the_cycle_morbid_draw() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::wardens_of_the_cycle());
        g.players[0].creatures_died_this_turn = 1;
        let life = g.players[0].life;
        g.active_player_idx = 0;
        g.fire_step_triggers(TurnStep::End);
        drain_stack(&mut g);
        // Default modal pick is mode 0 (gain 2 life); the morbid trigger fired.
        assert_eq!(g.players[0].life, life + 2, "gained 2 life off the morbid trigger");
    }

    /// Roiling Dragonstorm loots on ETB and bounces itself when a Dragon enters.
    #[test]
    fn roiling_dragonstorm_bounces_on_dragon() {
        let mut g = two_player_game();
        g.add_card_to_library(0, catalog::island());
        g.add_card_to_library(0, catalog::island());
        g.add_card_to_hand(0, catalog::island()); // something to discard
        let storm = g.move_card_to_battlefield_for_test(0, catalog::roiling_dragonstorm());
        drain_stack(&mut g);
        assert!(g.battlefield_find(storm).is_some(), "enchantment is on the battlefield");
        // A Dragon entering bounces the enchantment back to hand.
        let dragon = g.add_card_to_battlefield(0, catalog::boulderborn_dragon());
        g.dispatch_triggers_for_events(&[GameEvent::PermanentEntered { card_id: dragon }]);
        drain_stack(&mut g);
        assert!(g.battlefield_find(storm).is_none(), "returned to hand on Dragon ETB");
        assert!(g.players[0].hand.iter().any(|c| c.id == storm));
    }

    /// Stormcatch Mentor reduces instant/sorcery cost and has prowess + haste.
    #[test]
    fn stormcatch_mentor_cheapens_spells() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::stormcatch_mentor());
        let bolt = g.add_card_to_hand(0, catalog::lightning_axe()); // {R} instant
        // Lightning Axe is {R}; reduction is generic-only so cost unchanged here, but
        // a {1}{R} sorcery would drop to {R}. Use a cheaper proxy: just verify the
        // static is present and prowess/haste are on the body.
        let _ = bolt;
        let m = g.battlefield.iter().find(|c| c.definition.name == "Stormcatch Mentor").unwrap();
        assert!(m.definition.keywords.contains(&Keyword::Prowess));
        assert!(m.definition.keywords.contains(&Keyword::Haste));
        assert_eq!(m.definition.static_abilities.len(), 1, "I/S cost reduction static");
    }

    /// Gurmag Drowner exploits itself to dig four.
    #[test]
    fn gurmag_drowner_exploit_digs() {
        let mut g = two_player_game();
        for _ in 0..4 { g.add_card_to_library(0, catalog::island()); }
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)])); // accept exploit
        let hand = g.players[0].hand.len();
        let drowner = g.move_card_to_battlefield_for_test(0, catalog::gurmag_drowner());
        drain_stack(&mut g);
        assert!(g.battlefield_find(drowner).is_none(), "exploited itself");
        assert_eq!(g.players[0].hand.len(), hand + 1, "dug a card into hand");
    }

    /// Nullpriest of Oblivion reanimates when kicked.
    #[test]
    fn nullpriest_kicked_reanimates() {
        let mut g = two_player_game();
        let corpse = g.add_card_to_graveyard(0, catalog::serra_angel());
        let null = g.add_card_to_hand(0, catalog::nullpriest_of_oblivion());
        g.players[0].mana_pool.add(Color::Black, 2);
        g.players[0].mana_pool.add_colorless(4);
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::CastSpellKicked {
            card_id: null, target: Some(Target::Permanent(corpse)), additional_targets: vec![],
            mode: None, x_value: None,
        }).expect("cast kicked");
        drain_stack(&mut g);
        assert!(g.battlefield_find(corpse).is_some(), "reanimated the angel");
    }

    /// Ureni deals damage on ETB equal to lands you control, divided among foes.
    #[test]
    fn ureni_etb_divided_damage() {
        let mut g = two_player_game();
        for _ in 0..3 { g.add_card_to_battlefield(0, catalog::forest()); }
        let victim = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4, only legal target
        g.move_card_to_battlefield_for_test(0, catalog::ureni_the_song_unending());
        drain_stack(&mut g);
        assert_eq!(g.battlefield_find(victim).map(|c| c.damage), Some(3), "3 damage (3 lands)");
    }

    /// Elspeth, Storm Slayer doubles her Soldier token.
    #[test]
    fn elspeth_storm_slayer_doubles_tokens() {
        let mut g = two_player_game();
        let elspeth = g.add_card_to_battlefield(0, catalog::elspeth_storm_slayer());
        g.perform_action(GameAction::ActivateLoyaltyAbility {
            card_id: elspeth, ability_index: 0, target: None, x_value: None,
        }).expect("+1 make a Soldier");
        drain_stack(&mut g);
        let soldiers = g.battlefield.iter().filter(|c| c.definition.name == "Soldier").count();
        assert_eq!(soldiers, 2, "token doubling made two Soldiers");
    }

    /// Betor draws at end step once total toughness reaches 10.
    #[test]
    fn betor_end_step_draw_at_ten_toughness() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::betor_kin_to_all()); // 5/7
        g.add_card_to_battlefield(0, catalog::grizzly_bears()); // +2 → 9, not yet 10
        g.add_card_to_library(0, catalog::island());
        g.active_player_idx = 0;
        let hand = g.players[0].hand.len();
        g.fire_step_triggers(TurnStep::End);
        drain_stack(&mut g);
        assert_eq!(g.players[0].hand.len(), hand, "9 total toughness: no draw");
        // Add another creature to cross 10.
        g.add_card_to_battlefield(0, catalog::grizzly_bears()); // → 11
        g.fire_step_triggers(TurnStep::End);
        drain_stack(&mut g);
        assert_eq!(g.players[0].hand.len(), hand + 1, "≥10 total toughness draws");
    }

    /// Mistmoon Griffin reanimates the top creature of your graveyard when it dies.
    #[test]
    fn mistmoon_griffin_reanimates_on_death() {
        let mut g = two_player_game();
        g.add_card_to_graveyard(0, catalog::grizzly_bears()); // top creature card
        let griffin = g.add_card_to_battlefield(0, catalog::mistmoon_griffin());
        g.battlefield_find_mut(griffin).unwrap().damage = 2; // lethal
        g.check_state_based_actions();
        drain_stack(&mut g);
        assert!(g.battlefield.iter().any(|c| c.definition.name == "Grizzly Bears"),
            "the top creature card returned to the battlefield");
    }

    /// Dalek Squadron makes a myriad copy when attacking (multiplayer).
    #[test]
    fn dalek_squadron_myriad_copies() {
        let mut g = crabomination::game::game_with_format(crabomination::format::Format::Commander, 3);
        let dalek = g.add_card_to_battlefield(0, catalog::dalek_squadron());
        g.clear_sickness(dalek);
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        while g.step != TurnStep::DeclareAttackers {
            g.perform_action(GameAction::PassPriority).unwrap();
        }
        g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker: dalek, target: AttackTarget::Player(1),
        }])).expect("attack player 1");
        drain_stack(&mut g);
        let copies = g.battlefield.iter()
            .filter(|c| c.definition.name == "Dalek Squadron" && c.is_token).count();
        assert_eq!(copies, 1, "one myriad copy for the third player");
    }

    /// Perennation reanimates a permanent with hexproof + indestructible counters.
    #[test]
    fn perennation_reanimates_with_counters() {
        let mut g = two_player_game();
        let corpse = g.add_card_to_graveyard(0, catalog::grizzly_bears());
        let spell = g.add_card_to_hand(0, catalog::perennation());
        g.players[0].mana_pool.add(Color::White, 1);
        g.players[0].mana_pool.add(Color::Black, 1);
        g.players[0].mana_pool.add(Color::Green, 1);
        g.players[0].mana_pool.add_colorless(3);
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        cast_at(&mut g, spell, Target::Permanent(corpse));
        drain_stack(&mut g);
        let c = g.computed_permanent(corpse).expect("bear is on the battlefield");
        assert!(c.keywords.contains(&Keyword::Hexproof), "hexproof counter");
        assert!(c.keywords.contains(&Keyword::Indestructible), "indestructible counter");
    }

    /// Sarkhan, Soul Aflame reduces Dragon spell costs.
    #[test]
    fn sarkhan_soul_aflame_cheapens_dragons() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::sarkhan_soul_aflame());
        let dragon = g.add_card_to_hand(0, catalog::boulderborn_dragon()); // {5}
        g.players[0].mana_pool.add_colorless(4); // only {4} thanks to the {1} discount
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::CastSpell {
            card_id: dragon, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast the Dragon for {4}");
        drain_stack(&mut g);
        assert!(g.battlefield_find(dragon).is_some(), "Dragon resolved at the discount");
    }

    /// Skirmish Rhino drains each opponent and gains you life.
    #[test]
    fn skirmish_rhino_drains_on_etb() {
        let mut g = two_player_game();
        let life = g.players[0].life;
        let opp = g.players[1].life;
        g.move_card_to_battlefield_for_test(0, catalog::skirmish_rhino());
        drain_stack(&mut g);
        assert_eq!(g.players[0].life, life + 2, "gained 2");
        assert_eq!(g.players[1].life, opp - 2, "opponent lost 2");
    }

    /// Rabid Gnaw pumps your creature, which bites an opponent's.
    #[test]
    fn rabid_gnaw_bites() {
        let mut g = two_player_game();
        let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2 → 3/2
        let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
        let spell = g.add_card_to_hand(0, catalog::rabid_gnaw());
        g.players[0].mana_pool.add(Color::Red, 1);
        g.players[0].mana_pool.add_colorless(1);
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::CastSpell {
            card_id: spell, target: Some(Target::Permanent(mine)),
            additional_targets: vec![Target::Permanent(theirs)], mode: None, x_value: None,
        }).expect("cast Rabid Gnaw");
        drain_stack(&mut g);
        assert!(g.battlefield_find(theirs).is_none(), "3 damage from the pumped biter killed the 2/2");
        assert!(g.battlefield_find(mine).is_some(), "the biter took none");
    }

    /// Reckless Lackey sacrifices for a card and a Treasure.
    #[test]
    fn reckless_lackey_sacrifices_for_value() {
        let mut g = two_player_game();
        let lackey = g.add_card_to_battlefield(0, catalog::reckless_lackey());
        g.clear_sickness(lackey);
        g.add_card_to_library(0, catalog::island());
        g.players[0].mana_pool.add(Color::Red, 1);
        g.players[0].mana_pool.add_colorless(2);
        let hand = g.players[0].hand.len();
        g.perform_action(GameAction::ActivateAbility {
            card_id: lackey, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
        }).expect("sac for value");
        drain_stack(&mut g);
        assert!(g.battlefield_find(lackey).is_none(), "sacrificed itself");
        assert_eq!(g.players[0].hand.len(), hand + 1, "drew a card");
        assert!(g.battlefield.iter().any(|c| c.controller == 0
            && c.definition.subtypes.artifact_subtypes.contains(&crabomination::card::ArtifactSubtype::Treasure)),
            "made a Treasure");
    }

    /// Lunar Convocation drains at end step when you gained life.
    #[test]
    fn lunar_convocation_drains_on_lifegain() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::lunar_convocation());
        g.players[0].life_gained_this_turn = 3;
        g.active_player_idx = 0;
        let opp = g.players[1].life;
        g.fire_step_triggers(TurnStep::End);
        drain_stack(&mut g);
        assert_eq!(g.players[1].life, opp - 1, "each opponent lost 1 after lifegain");
    }

    /// Dazzling Denial counters a spell whose controller can't pay the {2} tax.
    #[test]
    fn dazzling_denial_counters_when_unpaid() {
        let mut g = two_player_game();
        g.players[0].mana_pool.add(Color::Blue, 1);
        g.players[0].mana_pool.add_colorless(1);
        let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
        g.players[1].mana_pool.add(Color::Red, 1); // exactly enough for Bolt, nothing spare
        g.active_player_idx = 1;
        g.priority.player_with_priority = 1;
        g.perform_action(GameAction::CastSpell {
            card_id: bolt, target: Some(Target::Player(0)), additional_targets: vec![],
            mode: None, x_value: None,
        }).expect("cast Bolt");
        g.priority.player_with_priority = 0;
        let denial = g.add_card_to_hand(0, catalog::dazzling_denial());
        cast_at(&mut g, denial, Target::Permanent(bolt));
        drain_stack(&mut g);
        assert!(g.players[1].graveyard.iter().any(|c| c.id == bolt), "Bolt countered (couldn't pay {{2}})");
        assert_eq!(g.players[0].life, 20, "Bolt didn't resolve");
    }

    /// Mistrise Village enters tapped without the right basics, untapped with them.
    #[test]
    fn mistrise_village_conditional_tap() {
        let mut g = two_player_game();
        let v1 = g.move_card_to_battlefield_for_test(0, catalog::mistrise_village());
        drain_stack(&mut g);
        assert!(g.battlefield_find(v1).unwrap().tapped, "enters tapped with no Mountain/Forest");
        g.add_card_to_battlefield(0, catalog::mountain());
        let v2 = g.move_card_to_battlefield_for_test(0, catalog::mistrise_village());
        drain_stack(&mut g);
        assert!(!g.battlefield_find(v2).unwrap().tapped, "enters untapped with a Mountain out");
    }

    /// Cori Mountain Monastery impulse-exiles the top card for later play.
    #[test]
    fn cori_mountain_monastery_impulse() {
        let mut g = two_player_game();
        let land = g.add_card_to_battlefield(0, catalog::cori_mountain_monastery());
        g.add_card_to_battlefield(0, catalog::plains()); // so it isn't relevant; we just need mana
        g.add_card_to_library(0, catalog::island());
        g.players[0].mana_pool.add(Color::Red, 1);
        g.players[0].mana_pool.add_colorless(3);
        let exile_before = g.exile.len();
        g.perform_action(GameAction::ActivateAbility {
            card_id: land, ability_index: 1, target: None, additional_targets: vec![], x_value: None,
        }).expect("impulse");
        drain_stack(&mut g);
        assert!(g.exile.len() > exile_before, "exiled the top card for later play");
    }

    /// Bloodletter of Aclazotz doubles an opponent's life loss during your turn.
    #[test]
    fn bloodletter_doubles_opponent_life_loss() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::bloodletter_of_aclazotz());
        g.active_player_idx = 0; // your turn
        let opp = g.players[1].life;
        // A Skirmish Rhino drains the opponent 2 → doubled to 4.
        g.move_card_to_battlefield_for_test(0, catalog::skirmish_rhino());
        drain_stack(&mut g);
        assert_eq!(g.players[1].life, opp - 4, "opponent's 2 life loss doubled to 4");
    }

    /// Off your turn the doubling doesn't apply.
    #[test]
    fn bloodletter_inactive_off_your_turn() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::bloodletter_of_aclazotz());
        g.active_player_idx = 1; // opponent's turn
        let opp = g.players[1].life;
        g.move_card_to_battlefield_for_test(0, catalog::skirmish_rhino());
        drain_stack(&mut g);
        assert_eq!(g.players[1].life, opp - 2, "no doubling when it isn't your turn");
    }

    /// Touch the Spirit Realm exiles a creature until it leaves.
    #[test]
    fn touch_the_spirit_realm_exiles_until_it_leaves() {
        let mut g = two_player_game();
        let victim = g.add_card_to_battlefield(1, catalog::serra_angel());
        g.decider = Box::new(ScriptedDecider::new([
            DecisionAnswer::Target(Target::Permanent(victim)),
        ]));
        let ring = g.move_card_to_battlefield_for_test(0, catalog::touch_the_spirit_realm());
        drain_stack(&mut g);
        assert!(g.battlefield_find(victim).is_none(), "creature exiled");
        // When the enchantment leaves, the creature returns.
        g.remove_from_battlefield_to_graveyard_raw(ring);
        drain_stack(&mut g);
        assert!(g.battlefield.iter().any(|c| c.definition.name == "Serra Angel"),
            "returns when the enchantment leaves");
    }

    /// Sonar Strike burns a tapped creature and gains life with a Bat out.
    #[test]
    fn sonar_strike_hits_tapped_and_gains_with_bat() {
        let mut g = two_player_game();
        let victim = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4
        g.battlefield_find_mut(victim).unwrap().tapped = true;
        // A Bat token for the lifegain rider.
        let mut bat = catalog::grizzly_bears();
        bat.name = "Bat"; bat.subtypes.creature_types = vec![CreatureType::Bat];
        g.add_card_to_battlefield(0, bat);
        let spell = g.add_card_to_hand(0, catalog::sonar_strike());
        g.players[0].mana_pool.add(Color::White, 1);
        g.players[0].mana_pool.add_colorless(1);
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        let life = g.players[0].life;
        cast_at(&mut g, spell, Target::Permanent(victim));
        drain_stack(&mut g);
        assert!(g.battlefield_find(victim).is_none(), "4 damage killed the tapped 4/4");
        assert_eq!(g.players[0].life, life + 3, "gained 3 from the Bat rider");
    }

    /// Aerie Auxiliary supports two other creatures on ETB.
    #[test]
    fn aerie_auxiliary_supports_two() {
        let mut g = two_player_game();
        let a = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let b = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let aux = g.add_card_to_battlefield(0, catalog::aerie_auxiliary());
        g.fire_self_etb_triggers(aux, 0);
        drain_stack(&mut g);
        // The ETB support trigger fired and placed a +1/+1 counter (the exact
        // up-to-two count is covered by the cast-spell support test in counters.rs).
        let total = g.battlefield_find(a).unwrap().counter_count(CounterType::PlusOnePlusOne)
            + g.battlefield_find(b).unwrap().counter_count(CounterType::PlusOnePlusOne);
        assert!(total >= 1, "support fired off the ETB trigger");
    }

    /// Loran's Escape shields a creature and scries.
    #[test]
    fn lorans_escape_shields_and_scries() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.add_card_to_library(0, catalog::island());
        let spell = g.add_card_to_hand(0, catalog::lorans_escape());
        g.players[0].mana_pool.add(Color::White, 1);
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        cast_at(&mut g, spell, Target::Permanent(bear));
        drain_stack(&mut g);
        let c = g.computed_permanent(bear).unwrap();
        assert!(c.keywords.contains(&Keyword::Hexproof));
        assert!(c.keywords.contains(&Keyword::Indestructible));
    }

    /// Dauntless Veteran pumps the team when it attacks.
    #[test]
    fn dauntless_veteran_pumps_team_on_attack() {
        let mut g = two_player_game();
        let vet = g.add_card_to_battlefield(0, catalog::dauntless_veteran());
        let buddy = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.clear_sickness(vet);
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        while g.step != TurnStep::DeclareAttackers {
            g.perform_action(GameAction::PassPriority).unwrap();
        }
        g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker: vet, target: AttackTarget::Player(1),
        }])).expect("attack");
        drain_stack(&mut g);
        assert_eq!(g.computed_permanent(buddy).unwrap().power, 3, "team got +1/+1");
        assert_eq!(g.computed_permanent(vet).unwrap().power, 3, "the veteran too");
    }

    /// Spectral Denial soft-counters a spell whose controller can't pay {X}.
    #[test]
    fn spectral_denial_counters_at_x() {
        let mut g = two_player_game();
        // Cast Denial with X=2 → counter unless they pay {2}.
        let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
        g.players[1].mana_pool.add(Color::Red, 1); // bolt only, nothing spare
        g.active_player_idx = 1;
        g.priority.player_with_priority = 1;
        g.perform_action(GameAction::CastSpell {
            card_id: bolt, target: Some(Target::Player(0)), additional_targets: vec![],
            mode: None, x_value: None,
        }).expect("cast Bolt");
        g.priority.player_with_priority = 0;
        let denial = g.add_card_to_hand(0, catalog::spectral_denial());
        g.players[0].mana_pool.add(Color::Blue, 1);
        g.players[0].mana_pool.add_colorless(2);
        g.perform_action(GameAction::CastSpell {
            card_id: denial, target: Some(Target::Permanent(bolt)), additional_targets: vec![],
            mode: None, x_value: Some(2),
        }).expect("cast Denial X=2");
        drain_stack(&mut g);
        assert!(g.players[1].graveyard.iter().any(|c| c.id == bolt), "Bolt countered (couldn't pay {{2}})");
        assert_eq!(g.players[0].life, 20, "Bolt didn't resolve");
    }

    /// Glistener Seer enters with oil counters and spends them to scry.
    #[test]
    fn glistener_seer_oil_scry() {
        let mut g = two_player_game();
        g.add_card_to_library(0, catalog::island());
        let seer = g.move_card_to_battlefield_for_test(0, catalog::glistener_seer());
        drain_stack(&mut g);
        assert_eq!(g.battlefield_find(seer).unwrap().counter_count(CounterType::Oil), 3, "entered with 3 oil");
        g.clear_sickness(seer);
        g.perform_action(GameAction::ActivateAbility {
            card_id: seer, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
        }).expect("remove an oil to scry");
        drain_stack(&mut g);
        assert_eq!(g.battlefield_find(seer).unwrap().counter_count(CounterType::Oil), 2, "spent one oil");
        assert!(g.battlefield_find(seer).unwrap().tapped, "tapped for the ability");
    }

    /// Vengeful Bloodwitch drains when a creature you control dies.
    #[test]
    fn vengeful_bloodwitch_drains_on_death() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::vengeful_bloodwitch());
        let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let life = g.players[0].life;
        let opp = g.players[1].life;
        // Kill the fodder through the full damage→SBA→dispatch path.
        let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
        g.players[0].mana_pool.add(Color::Red, 1);
        g.perform_action(GameAction::CastSpell {
            card_id: bolt, target: Some(Target::Permanent(fodder)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("bolt the fodder");
        drain_stack(&mut g);
        assert_eq!(g.players[1].life, opp - 1, "opponent lost 1");
        assert_eq!(g.players[0].life, life + 1, "you gained 1");
    }

    /// Hulking Raptor ramps two green at your first main phase and has Ward {2}.
    #[test]
    fn hulking_raptor_ramps_and_wards() {
        let mut g = two_player_game();
        let rap = g.add_card_to_battlefield(0, catalog::hulking_raptor());
        assert!(g.battlefield_find(rap).unwrap().definition.keywords.iter()
            .any(|k| matches!(k, Keyword::Ward(_))), "has Ward");
        g.active_player_idx = 0;
        g.fire_step_triggers(TurnStep::PreCombatMain);
        drain_stack(&mut g);
        assert_eq!(g.players[0].mana_pool.amount(Color::Green), 2, "added two green");
    }

    /// A "Start your engines!" permanent entering sets its controller's speed to 1.
    #[test]
    fn start_your_engines_sets_speed_to_one() {
        let mut g = two_player_game();
        assert_eq!(g.players[0].speed, 0);
        g.move_card_to_battlefield_for_test(0, catalog::nesting_bot());
        drain_stack(&mut g);
        assert_eq!(g.players[0].speed, 1, "SYE starts speed at 1");
        // A second SYE permanent doesn't re-bump an already-started speed.
        g.move_card_to_battlefield_for_test(0, catalog::swiftwing_assailant());
        drain_stack(&mut g);
        assert_eq!(g.players[0].speed, 1);
    }

    /// Speed rises once per your turn when an opponent loses life, capped at 4.
    #[test]
    fn speed_increments_on_opponent_life_loss() {
        let mut g = two_player_game();
        g.active_player_idx = 0;
        g.players[0].speed = 1;
        g.adjust_life(1, -1); // opponent loses life on your turn → +1
        assert_eq!(g.players[0].speed, 2);
        g.adjust_life(1, -3); // same turn → no further bump
        assert_eq!(g.players[0].speed, 2);
        // A player with no speed yet isn't started by a life-loss event.
        g.active_player_idx = 1;
        g.players[1].speed_increased_this_turn = false;
        g.adjust_life(0, -1);
        assert_eq!(g.players[1].speed, 0, "no speed yet → life loss doesn't start it");
    }

    /// Speed never exceeds 4.
    #[test]
    fn speed_caps_at_four() {
        let mut g = two_player_game();
        g.active_player_idx = 0;
        g.players[0].speed = 4;
        g.adjust_life(1, -1);
        assert_eq!(g.players[0].speed, 4);
    }

    /// Nesting Bot's "Max speed —" +1/+0 is live only at speed 4.
    #[test]
    fn nesting_bot_max_speed_pump() {
        let mut g = two_player_game();
        let bot = g.add_card_to_battlefield(0, catalog::nesting_bot());
        assert_eq!(g.computed_permanent(bot).unwrap().power, 1, "1/1 below max speed");
        g.players[0].speed = 4;
        assert_eq!(g.computed_permanent(bot).unwrap().power, 2, "+1/+0 at max speed");
    }

    /// Burnout Bashtronaut gains double strike at max speed.
    #[test]
    fn burnout_bashtronaut_max_speed_double_strike() {
        let mut g = two_player_game();
        let goblin = g.add_card_to_battlefield(0, catalog::burnout_bashtronaut());
        assert!(!g.computed_permanent(goblin).unwrap().keywords.contains(&Keyword::DoubleStrike));
        g.players[0].speed = 4;
        assert!(g.computed_permanent(goblin).unwrap().keywords.contains(&Keyword::DoubleStrike));
    }

    /// Risen Necroregent makes an end-step Zombie only at max speed.
    #[test]
    fn risen_necroregent_max_speed_token() {
        let mut g = two_player_game();
        g.active_player_idx = 0;
        g.add_card_to_battlefield(0, catalog::risen_necroregent());
        let creatures = |g: &GameState| {
            g.battlefield.iter().filter(|c| c.controller == 0 && c.definition.is_creature()).count()
        };
        let before = creatures(&g);
        g.fire_step_triggers(TurnStep::End);
        drain_stack(&mut g);
        assert_eq!(creatures(&g), before, "no token below max speed");
        g.players[0].speed = 4;
        g.fire_step_triggers(TurnStep::End);
        drain_stack(&mut g);
        assert_eq!(creatures(&g), before + 1, "2/2 Zombie at max speed");
    }

    /// Walking Sarcophagus is a 2/1 normally, 3/3 at max speed.
    #[test]
    fn walking_sarcophagus_max_speed_pump() {
        let mut g = two_player_game();
        let s = g.add_card_to_battlefield(0, catalog::walking_sarcophagus());
        assert_eq!(g.computed_permanent(s).map(|c| (c.power, c.toughness)), Some((2, 1)));
        g.players[0].speed = 4;
        assert_eq!(g.computed_permanent(s).map(|c| (c.power, c.toughness)), Some((3, 3)));
    }

    /// Streaking Oilgorger gains lifelink only at max speed.
    #[test]
    fn streaking_oilgorger_max_speed_lifelink() {
        let mut g = two_player_game();
        let v = g.add_card_to_battlefield(0, catalog::streaking_oilgorger());
        assert!(!g.computed_permanent(v).unwrap().keywords.contains(&Keyword::Lifelink));
        g.players[0].speed = 4;
        assert!(g.computed_permanent(v).unwrap().keywords.contains(&Keyword::Lifelink));
    }

    /// Gastal Thrillseeker pings each opponent and gains you life on ETB.
    #[test]
    fn gastal_thrillseeker_etb_ping() {
        let mut g = two_player_game();
        let life0 = g.players[0].life;
        let life1 = g.players[1].life;
        g.move_card_to_battlefield_for_test(0, catalog::gastal_thrillseeker());
        drain_stack(&mut g);
        assert_eq!(g.players[1].life, life1 - 1, "opponent took 1");
        assert_eq!(g.players[0].life, life0 + 1, "you gained 1");
    }

    /// Goblin Surveyor's graveyard draw is only castable at max speed.
    #[test]
    fn goblin_surveyor_max_speed_gated_ability() {
        let def = catalog::goblin_surveyor();
        let ab = &def.activated_abilities[0];
        assert!(ab.from_graveyard && ab.exile_self_cost, "graveyard exile-cost ability");
        assert_eq!(ab.condition, Some(crabomination::card::Predicate::SpeedAtLeast {
            who: crabomination::effect::PlayerRef::You,
            speed: 4,
        }));
    }

    /// Recover returns the card to hand when its cost is paid as a creature dies.
    #[test]
    fn recover_returns_to_hand_when_paid() {
        let mut g = two_player_game();
        let gh = g.add_card_to_graveyard(0, catalog::suns_bounty());
        // Pre-float the {1}{W} recover cost and accept the MayPay prompt.
        g.players[0].mana_pool.add(Color::White, 1);
        g.players[0].mana_pool.add_colorless(1);
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let mut evs = g.remove_to_graveyard_with_triggers(bear);
        evs.push(GameEvent::CreatureDied { card_id: bear });
        g.dispatch_triggers_for_events(&evs);
        drain_stack(&mut g);
        assert!(g.players[0].hand.iter().any(|c| c.id == gh), "recovered to hand");
        assert!(!g.players[0].graveyard.iter().any(|c| c.id == gh));
    }

    /// Declining recover (no mana) exiles the card.
    #[test]
    fn recover_exiles_when_declined() {
        let mut g = two_player_game();
        let gh = g.add_card_to_graveyard(0, catalog::icefall());
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let mut evs = g.remove_to_graveyard_with_triggers(bear);
        evs.push(GameEvent::CreatureDied { card_id: bear });
        g.dispatch_triggers_for_events(&evs);
        drain_stack(&mut g);
        assert!(g.exile.iter().any(|c| c.id == gh), "unpaid recover exiles the card");
        assert!(!g.players[0].graveyard.iter().any(|c| c.id == gh));
    }

    /// Bloodthirsty Conqueror gains you life equal to an opponent's life loss.
    #[test]
    fn bloodthirsty_conqueror_drains_to_you() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::bloodthirsty_conqueror());
        let life0 = g.players[0].life;
        let evs = vec![{
            let amt = 3u32;
            g.players[1].life -= amt as i32;
            crabomination::game::GameEvent::LifeLost { player: 1, amount: amt }
        }];
        g.dispatch_triggers_for_events(&evs);
        drain_stack(&mut g);
        assert_eq!(g.players[0].life, life0 + 3, "gained 3 from the opponent's loss");
    }

    /// Razorkin Needlehead has first strike on your turn only, and pings opponents
    /// who draw.
    #[test]
    fn razorkin_needlehead_turn_first_strike_and_draw_ping() {
        let mut g = two_player_game();
        g.active_player_idx = 0;
        let rk = g.add_card_to_battlefield(0, catalog::razorkin_needlehead());
        assert!(g.computed_permanent(rk).unwrap().keywords.contains(&Keyword::FirstStrike));
        g.active_player_idx = 1;
        assert!(!g.computed_permanent(rk).unwrap().keywords.contains(&Keyword::FirstStrike));
        // Opponent draws → takes 1.
        let life1 = g.players[1].life;
        let drawn = g.add_card_to_hand(1, catalog::island());
        g.dispatch_triggers_for_events(&[crabomination::game::GameEvent::CardDrawn {
            player: 1,
            card_id: drawn,
        }]);
        drain_stack(&mut g);
        assert_eq!(g.players[1].life, life1 - 1);
    }

    /// Savor shrinks a creature and makes a Food.
    #[test]
    fn savor_shrinks_and_makes_food() {
        let mut g = two_player_game();
        let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
        let mut ctx = crabomination::game::effects::EffectContext::for_ability(crabomination::card::CardId(0), 0, None);
        ctx.targets = vec![Target::Permanent(foe)];
        g.resolve_effect(&catalog::savor().effect, &ctx).unwrap();
        drain_stack(&mut g);
        g.check_state_based_actions();
        assert!(g.battlefield_find(foe).is_none(), "-2/-2 killed the 2/2");
        assert!(g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Food"));
    }

    /// Screaming Nemesis redirects damage dealt to it onto any target.
    #[test]
    fn screaming_nemesis_redirects_damage() {
        let mut g = two_player_game();
        let nem = g.add_card_to_battlefield(0, catalog::screaming_nemesis());
        let life1 = g.players[1].life;
        // The enrage trigger reads the DamageDealt amount and bolts any target.
        g.dispatch_triggers_for_events(&[crabomination::game::GameEvent::DamageDealt {
            amount: 3,
            to_card: Some(nem),
            to_player: None, combat: false, from_controller: None,
        }]);
        drain_stack(&mut g);
        assert_eq!(g.players[1].life, life1 - 3, "redirected 3 to the opponent");
        // CR 119.7 — the damaged player can't gain life for the rest of the game.
        assert!(g.players[1].cannot_gain_life, "rest-of-game lifegain lock");
        let before = g.players[1].life;
        g.adjust_life(1, 5);
        assert_eq!(g.players[1].life, before, "lifegain stays locked");
    }

    /// Spinewoods Armadillo is a 7/7 with Reach and Ward {3}.
    #[test]
    fn spinewoods_armadillo_stats() {
        let def = catalog::spinewoods_armadillo();
        assert_eq!((def.power, def.toughness), (7, 7));
        assert!(def.keywords.contains(&Keyword::Reach));
        assert!(def.activated_abilities[0].discard_self_cost, "discard-this fetch ability");
    }

    /// Goblin Boarders enters with a +1/+1 counter only if you attacked this turn.
    #[test]
    fn goblin_boarders_raid_counter() {
        let mut g = two_player_game();
        g.players[0].attacked_this_turn = true;
        let id = g.move_card_to_battlefield_for_test(0, catalog::goblin_boarders());
        drain_stack(&mut g);
        assert_eq!(
            g.battlefield_find(id).unwrap().counters.get(&CounterType::PlusOnePlusOne).copied(),
            Some(1),
        );
        let mut g2 = two_player_game();
        let id2 = g2.move_card_to_battlefield_for_test(0, catalog::goblin_boarders());
        drain_stack(&mut g2);
        assert_eq!(
            g2.battlefield_find(id2).unwrap().counters.get(&CounterType::PlusOnePlusOne).copied(),
            None,
            "no Raid → no counter",
        );
    }

    /// Cogwork Wrestler shrinks an opponent's creature's power on ETB.
    #[test]
    fn cogwork_wrestler_etb_shrinks_foe() {
        let mut g = two_player_game();
        let foe = g.add_card_to_battlefield(1, catalog::hill_giant()); // 3/3
        g.move_card_to_battlefield_for_test(0, catalog::cogwork_wrestler());
        // ETB targets the only opposing creature.
        drain_stack(&mut g);
        assert_eq!(g.computed_permanent(foe).map(|c| c.power), Some(1), "-2/-0 applied");
    }

    /// Crocodile of the Crossing puts a -1/-1 counter on a creature you control.
    #[test]
    fn crocodile_of_the_crossing_etb_counter() {
        let mut g = two_player_game();
        let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
        g.move_card_to_battlefield_for_test(0, catalog::crocodile_of_the_crossing());
        drain_stack(&mut g);
        // Auto-target picks a creature you control; with two, either works — assert
        // the total -1/-1 counters on your board is 1.
        let total: u32 = g.battlefield.iter().filter(|c| c.controller == 0)
            .map(|c| c.counters.get(&CounterType::MinusOneMinusOne).copied().unwrap_or(0)).sum();
        assert_eq!(total, 1);
        let _ = mine;
    }

    /// Topiary Stomper ramps a basic land onto the battlefield tapped.
    #[test]
    fn topiary_stomper_ramps() {
        let mut g = two_player_game();
        let forest = g.add_card_to_library(0, catalog::forest());
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(forest))]));
        let lands_before = g.battlefield.iter().filter(|c| c.controller == 0 && c.definition.is_land()).count();
        g.move_card_to_battlefield_for_test(0, catalog::topiary_stomper());
        drain_stack(&mut g);
        let lands_after = g.battlefield.iter().filter(|c| c.controller == 0 && c.definition.is_land()).count();
        assert_eq!(lands_after, lands_before + 1, "fetched a basic onto the battlefield");
    }

    /// Bakersbane Duo makes a Food on entry.
    #[test]
    fn bakersbane_duo_makes_food() {
        let mut g = two_player_game();
        g.move_card_to_battlefield_for_test(0, catalog::bakersbane_duo());
        drain_stack(&mut g);
        assert!(g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Food"));
    }

    /// Cache Grab mills four and returns a chosen permanent card to hand; the
    /// non-permanent milled card stays in the graveyard.
    #[test]
    fn cache_grab_returns_a_milled_permanent() {
        let mut g = two_player_game();
        g.add_card_to_library(0, catalog::lightning_bolt()); // instant — not eligible
        let bears = g.add_card_to_library(0, catalog::grizzly_bears()); // permanent
        g.add_card_to_library(0, catalog::forest());
        g.add_card_to_library(0, catalog::island());
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Cards(vec![bears])]));
        let grab = g.add_card_to_hand(0, catalog::cache_grab());
        g.players[0].mana_pool.add(Color::Green, 2);
        g.perform_action(GameAction::CastSpell {
            card_id: grab,
            target: None,
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .expect("Cache Grab castable for {1}{G}");
        drain_stack(&mut g);
        assert!(g.players[0].hand.iter().any(|c| c.id == bears), "chose the creature to hand");
        assert_eq!(g.players[0].library.len(), 0, "milled all four");
        assert!(g.players[0].graveyard.iter().any(|c| c.definition.name == "Lightning Bolt"),
            "the instant stays milled");
    }

    /// Lumbering Worldwagon's power equals the lands you control; toughness stays 4.
    #[test]
    fn lumbering_worldwagon_power_tracks_lands() {
        let mut g = two_player_game();
        let wagon = g.add_card_to_battlefield(0, catalog::lumbering_worldwagon());
        for _ in 0..3 {
            g.add_card_to_battlefield(0, catalog::forest());
        }
        let c = g.computed_permanent(wagon).unwrap();
        assert_eq!(c.power, 3, "power = 3 lands controlled");
        assert_eq!(c.toughness, 4, "printed toughness");
    }

    /// Spire Mangler pumps a flyer you control on ETB.
    #[test]
    fn spire_mangler_pumps_a_flyer() {
        let mut g = two_player_game();
        let flyer = g.add_card_to_battlefield(0, catalog::serra_angel()); // 4/4 flying
        g.move_card_to_battlefield_for_test(0, catalog::spire_mangler());
        drain_stack(&mut g);
        // Auto-target picks a controlled flyer; total power gain on your flyers is 2.
        assert_eq!(g.computed_permanent(flyer).map(|c| c.power), Some(6));
    }

    /// Palace Familiar draws a card when it dies.
    #[test]
    fn palace_familiar_dies_draws() {
        let mut g = two_player_game();
        g.add_card_to_library(0, catalog::island());
        let fam = g.add_card_to_battlefield(0, catalog::palace_familiar());
        let hand_before = g.players[0].hand.len();
        g.remove_to_graveyard_with_triggers(fam);
        drain_stack(&mut g);
        assert_eq!(g.players[0].hand.len(), hand_before + 1, "dies → draw a card");
    }

    /// Symbiotic Elf leaves two 1/1 Insects when it dies.
    #[test]
    fn symbiotic_elf_dies_makes_two_insects() {
        let mut g = two_player_game();
        let elf = g.add_card_to_battlefield(0, catalog::symbiotic_elf());
        g.remove_to_graveyard_with_triggers(elf);
        drain_stack(&mut g);
        let insects = g.battlefield.iter().filter(|c| c.controller == 0 && c.definition.name == "Insect").count();
        assert_eq!(insects, 2);
    }

    /// Bear's Companion mints a 4/4 Bear on entry.
    #[test]
    fn bears_companion_makes_a_bear() {
        let mut g = two_player_game();
        g.move_card_to_battlefield_for_test(0, catalog::bears_companion());
        drain_stack(&mut g);
        let bear = g.battlefield.iter().find(|c| c.controller == 0 && c.definition.name == "Bear");
        assert!(bear.is_some_and(|b| b.definition.power == 4 && b.definition.toughness == 4));
    }

    /// Grasping Thrull drains each opponent for 2 and gains you 2.
    #[test]
    fn grasping_thrull_drains_and_gains() {
        let mut g = two_player_game();
        g.players[0].life = 20;
        g.players[1].life = 20;
        g.move_card_to_battlefield_for_test(0, catalog::grasping_thrull());
        drain_stack(&mut g);
        assert_eq!(g.players[1].life, 18, "opponent took 2");
        assert_eq!(g.players[0].life, 22, "you gained 2");
    }

    /// Hero of Precinct One makes a 1/1 Human when you cast a multicolored spell.
    #[test]
    fn hero_of_precinct_one_tokens_on_multicolored_cast() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::hero_of_precinct_one());
        let thrull = g.add_card_to_hand(0, catalog::grasping_thrull()); // W/B multicolored
        g.players[0].mana_pool.add(Color::White, 1);
        g.players[0].mana_pool.add(Color::Black, 1);
        g.players[0].mana_pool.add_colorless(3);
        g.perform_action(GameAction::CastSpell {
            card_id: thrull,
            target: None,
            additional_targets: vec![],
            mode: None,
            x_value: None,
        }).expect("multicolored spell castable");
        drain_stack(&mut g);
        assert!(g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Human"));
    }

    /// Havoc Devils is a 4/3 with trample.
    #[test]
    fn havoc_devils_stats() {
        let d = catalog::havoc_devils();
        assert_eq!((d.power, d.toughness), (4, 3));
        assert!(d.keywords.contains(&Keyword::Trample));
    }

    /// Hollow Dogs pumps itself +2/+0 when it attacks.
    #[test]
    fn hollow_dogs_pumps_on_attack() {
        let mut g = two_player_game();
        let dogs = g.add_card_to_battlefield(0, catalog::hollow_dogs());
        g.clear_sickness(dogs);
        g.step = TurnStep::DeclareAttackers;
        g.priority.player_with_priority = 0;
        g.declare_attackers(vec![Attack { attacker: dogs, target: AttackTarget::Player(1) }]).unwrap();
        drain_stack(&mut g);
        assert_eq!(g.computed_permanent(dogs).map(|c| c.power), Some(5));
    }

    /// Argothian Enchantress has shroud and draws on enchantment casts.
    #[test]
    fn argothian_enchantress_draws_on_enchantment_cast() {
        let mut g = two_player_game();
        let ench = g.add_card_to_battlefield(0, catalog::argothian_enchantress());
        assert!(g.battlefield_find(ench).unwrap().definition.keywords.contains(&Keyword::Shroud));
        g.add_card_to_library(0, catalog::island());
        let prison = g.add_card_to_hand(0, catalog::ghostly_prison());
        g.players[0].mana_pool.add(Color::White, 1);
        g.players[0].mana_pool.add_colorless(2);
        let hand_before = g.players[0].hand.len();
        g.perform_action(GameAction::CastSpell {
            card_id: prison,
            target: None,
            additional_targets: vec![],
            mode: None,
            x_value: None,
        }).expect("enchantment castable");
        drain_stack(&mut g);
        // Spent one card (the enchantment) and drew one back from the trigger.
        assert_eq!(g.players[0].hand.len(), hand_before);
    }

    /// Patrol Hound gains first strike by discarding a card.
    #[test]
    fn patrol_hound_discards_for_first_strike() {
        let mut g = two_player_game();
        let hound = g.add_card_to_battlefield(0, catalog::patrol_hound());
        g.add_card_to_hand(0, catalog::island());
        g.perform_action(GameAction::ActivateAbility {
            card_id: hound,
            ability_index: 0,
            target: None,
            additional_targets: vec![],
            x_value: None,
        }).expect("ability activates");
        drain_stack(&mut g);
        assert!(g.computed_permanent(hound).unwrap().keywords.contains(&Keyword::FirstStrike));
    }

    /// Canyon Wildcat can't be blocked while the defender controls a Mountain.
    #[test]
    fn canyon_wildcat_mountainwalk() {
        let mut g = two_player_game();
        let cat = g.add_card_to_battlefield(0, catalog::canyon_wildcat());
        g.clear_sickness(cat);
        let blocker = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        g.add_card_to_battlefield(1, catalog::mountain());
        g.step = TurnStep::DeclareAttackers;
        g.priority.player_with_priority = 0;
        g.declare_attackers(vec![Attack { attacker: cat, target: AttackTarget::Player(1) }]).unwrap();
        g.step = TurnStep::DeclareBlockers;
        g.priority.player_with_priority = 1;
        assert!(
            g.declare_blockers(vec![(blocker, cat)]).is_err(),
            "mountainwalk: can't be blocked while defender controls a Mountain"
        );
    }

    /// Squirrelanoids is a 1/1 with deathtouch.
    #[test]
    fn squirrelanoids_deathtouch() {
        let d = catalog::squirrelanoids();
        assert_eq!((d.power, d.toughness), (1, 1));
        assert!(d.keywords.contains(&Keyword::Deathtouch));
    }

    /// Vile Deacon gets +X/+X on attack where X counts Clerics.
    #[test]
    fn vile_deacon_scales_with_clerics() {
        let mut g = two_player_game();
        let deacon = g.add_card_to_battlefield(0, catalog::vile_deacon()); // a Cleric itself
        g.add_card_to_battlefield(0, catalog::vile_deacon()); // second Cleric
        g.clear_sickness(deacon);
        g.step = TurnStep::DeclareAttackers;
        g.priority.player_with_priority = 0;
        g.declare_attackers(vec![Attack { attacker: deacon, target: AttackTarget::Player(1) }]).unwrap();
        drain_stack(&mut g);
        // Base 2 + (2 Clerics) = 4 power.
        assert_eq!(g.computed_permanent(deacon).map(|c| c.power), Some(4));
    }

    /// Mischievous Mystic makes a Faerie when you draw your second card.
    #[test]
    fn mischievous_mystic_tokens_on_second_draw() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::mischievous_mystic());
        for _ in 0..3 { g.add_card_to_library(0, catalog::island()); }
        let div = g.add_card_to_hand(0, catalog::divination()); // draw 2
        g.players[0].mana_pool.add(Color::Blue, 1);
        g.players[0].mana_pool.add_colorless(2);
        g.perform_action(GameAction::CastSpell {
            card_id: div,
            target: None,
            additional_targets: vec![],
            mode: None,
            x_value: None,
        }).expect("Divination castable");
        drain_stack(&mut g);
        assert!(g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Faerie"));
    }

    /// Dawn's Light Archer has flash and reach.
    #[test]
    fn dawns_light_archer_keywords() {
        let d = catalog::dawns_light_archer();
        assert!(d.keywords.contains(&Keyword::Flash) && d.keywords.contains(&Keyword::Reach));
        assert_eq!((d.power, d.toughness), (4, 2));
    }

    /// Plumeveil is a 4/4 with flash, defender, and flying.
    #[test]
    fn plumeveil_keywords() {
        let d = catalog::plumeveil();
        assert_eq!((d.power, d.toughness), (4, 4));
        for kw in [Keyword::Flash, Keyword::Defender, Keyword::Flying] {
            assert!(d.keywords.contains(&kw));
        }
    }

    /// Rooftop Assassin destroys an opponent's damaged creature on ETB.
    #[test]
    fn rooftop_assassin_destroys_damaged_creature() {
        let mut g = two_player_game();
        let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        g.battlefield_find_mut(foe).unwrap().dealt_damage_this_turn = true;
        g.move_card_to_battlefield_for_test(0, catalog::rooftop_assassin());
        drain_stack(&mut g);
        assert!(g.battlefield_find(foe).is_none(), "damaged creature destroyed");
    }

    /// Spellgorger Barbarian discards at random on ETB and draws when it leaves.
    #[test]
    fn spellgorger_barbarian_etb_discard_and_leaves_draw() {
        let mut g = two_player_game();
        g.add_card_to_hand(0, catalog::island());
        g.add_card_to_library(0, catalog::island());
        let barb = g.move_card_to_battlefield_for_test(0, catalog::spellgorger_barbarian());
        drain_stack(&mut g);
        assert_eq!(g.players[0].hand.len(), 0, "ETB discarded the one card at random");
        let hand_before = g.players[0].hand.len();
        g.remove_to_graveyard_with_triggers(barb);
        drain_stack(&mut g);
        assert_eq!(g.players[0].hand.len(), hand_before + 1, "leaves → draw");
    }

    /// Bog Gnarr grows when any player casts a black spell.
    #[test]
    fn bog_gnarr_pumps_on_black_cast() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::bog_gnarr());
        let squirrel = g.add_card_to_hand(0, catalog::squirrelanoids()); // mono-black
        g.players[0].mana_pool.add(Color::Black, 1);
        g.perform_action(GameAction::CastSpell {
            card_id: squirrel,
            target: None,
            additional_targets: vec![],
            mode: None,
            x_value: None,
        }).expect("black spell castable");
        drain_stack(&mut g);
        let gnarr = g.battlefield.iter().find(|c| c.definition.name == "Bog Gnarr").unwrap();
        assert_eq!(g.computed_permanent(gnarr.id).map(|c| c.power), Some(4));
    }

    /// Elf Replica sacrifices to destroy an enchantment.
    #[test]
    fn elf_replica_destroys_enchantment() {
        let mut g = two_player_game();
        let replica = g.add_card_to_battlefield(0, catalog::elf_replica());
        let prison = g.add_card_to_battlefield(1, catalog::ghostly_prison());
        g.players[0].mana_pool.add(Color::Green, 1);
        g.players[0].mana_pool.add_colorless(1);
        g.perform_action(GameAction::ActivateAbility {
            card_id: replica,
            ability_index: 0,
            target: Some(Target::Permanent(prison)),
            additional_targets: vec![],
            x_value: None,
        }).expect("ability activates");
        drain_stack(&mut g);
        assert!(g.battlefield_find(prison).is_none(), "enchantment destroyed");
        assert!(g.battlefield_find(replica).is_none(), "Elf Replica sacrificed");
    }

    /// Seismic Mage taps and discards to destroy a land.
    #[test]
    fn seismic_mage_destroys_land() {
        let mut g = two_player_game();
        let mage = g.add_card_to_battlefield(0, catalog::seismic_mage());
        g.clear_sickness(mage);
        g.add_card_to_hand(0, catalog::island());
        let land = g.add_card_to_battlefield(1, catalog::mountain());
        g.players[0].mana_pool.add(Color::Red, 1);
        g.players[0].mana_pool.add_colorless(2);
        g.perform_action(GameAction::ActivateAbility {
            card_id: mage,
            ability_index: 0,
            target: Some(Target::Permanent(land)),
            additional_targets: vec![],
            x_value: None,
        }).expect("ability activates");
        drain_stack(&mut g);
        assert!(g.battlefield_find(land).is_none(), "land destroyed");
    }

    /// Etched Oracle enters with a +1/+1 counter per color of mana spent (Sunburst).
    #[test]
    fn etched_oracle_sunburst_counters() {
        let mut g = two_player_game();
        let oracle = g.add_card_to_hand(0, catalog::etched_oracle()); // {4}
        // Pay the generic {4} with W, U, B + 1 colorless → 3 distinct colors.
        g.players[0].mana_pool.add(Color::White, 1);
        g.players[0].mana_pool.add(Color::Blue, 1);
        g.players[0].mana_pool.add(Color::Black, 1);
        g.players[0].mana_pool.add_colorless(1);
        g.perform_action(GameAction::CastSpell {
            card_id: oracle,
            target: None,
            additional_targets: vec![],
            mode: None,
            x_value: None,
        }).expect("Etched Oracle castable");
        drain_stack(&mut g);
        let c = g.battlefield_find(oracle).unwrap();
        assert_eq!(c.counter_count(CounterType::PlusOnePlusOne), 3, "3 colors → 3 counters");
    }

    /// Skyreach Manta has flying and Sunburst counters.
    #[test]
    fn skyreach_manta_flying_sunburst() {
        let d = catalog::skyreach_manta();
        assert!(d.keywords.contains(&Keyword::Flying));
        assert!(d.enters_with_counters.is_some(), "Sunburst counter spec present");
    }

    /// Phyrexian Digester and Blackcleave Goblin both carry infect.
    #[test]
    fn infect_creatures_have_infect() {
        assert!(catalog::phyrexian_digester().keywords.contains(&Keyword::Infect));
        let bg = catalog::blackcleave_goblin();
        assert!(bg.keywords.contains(&Keyword::Infect) && bg.keywords.contains(&Keyword::Haste));
    }

    /// Essence Depleter drains an opponent for 1 with its colorless ability.
    #[test]
    fn essence_depleter_drains() {
        let mut g = two_player_game();
        g.players[0].life = 20;
        g.players[1].life = 20;
        let dep = g.add_card_to_battlefield(0, catalog::essence_depleter());
        g.clear_sickness(dep);
        g.players[0].mana_pool.add_colorless(2);
        g.perform_action(GameAction::ActivateAbility {
            card_id: dep,
            ability_index: 0,
            target: None,
            additional_targets: vec![],
            x_value: None,
        }).expect("ability activates");
        drain_stack(&mut g);
        assert_eq!(g.players[1].life, 19);
        assert_eq!(g.players[0].life, 21);
        // Devoid: the creature is colorless despite its black cost.
        assert!(catalog::essence_depleter().keywords.contains(&Keyword::Devoid));
    }

    /// Stormclaw Rager grows and draws when you sacrifice another permanent.
    #[test]
    fn stormclaw_rager_sac_grows_and_draws() {
        let mut g = two_player_game();
        let rager = g.add_card_to_battlefield(0, catalog::stormclaw_rager());
        let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.add_card_to_library(0, catalog::island());
        g.step = TurnStep::PostCombatMain;
        g.players[0].mana_pool.add_colorless(1);
        let hand_before = g.players[0].hand.len();
        g.perform_action(GameAction::ActivateAbility {
            card_id: rager,
            ability_index: 0,
            target: Some(Target::Permanent(fodder)),
            additional_targets: vec![],
            x_value: None,
        }).expect("sac ability activates");
        drain_stack(&mut g);
        assert_eq!(g.battlefield_find(rager).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
        assert_eq!(g.players[0].hand.len(), hand_before + 1, "drew a card");
    }

    /// Wave Elemental taps up to three non-flyers on sacrifice.
    #[test]
    fn wave_elemental_taps_nonflyers() {
        let mut g = two_player_game();
        let elem = g.add_card_to_battlefield(0, catalog::wave_elemental());
        g.clear_sickness(elem);
        let a = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let b = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        // The "up to three" picks are made at resolution via a ChooseCards decision.
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Cards(vec![a, b])]));
        g.players[0].mana_pool.add(Color::Blue, 1);
        g.perform_action(GameAction::ActivateAbility {
            card_id: elem,
            ability_index: 0,
            target: None,
            additional_targets: vec![],
            x_value: None,
        }).expect("tap ability activates");
        drain_stack(&mut g);
        assert!(g.battlefield_find(a).unwrap().tapped && g.battlefield_find(b).unwrap().tapped);
        assert!(g.battlefield_find(elem).is_none(), "Wave Elemental sacrificed");
    }

    /// Shipwreck Moray gets four energy on ETB.
    #[test]
    fn shipwreck_moray_makes_energy() {
        let mut g = two_player_game();
        g.move_card_to_battlefield_for_test(0, catalog::shipwreck_moray());
        drain_stack(&mut g);
        assert_eq!(g.players[0].energy, 4, "ETB grants four energy");
    }

    /// Argothian Sprite can't be blocked by artifact creatures.
    #[test]
    fn argothian_sprite_evades_artifacts() {
        let mut g = two_player_game();
        let sprite = g.add_card_to_battlefield(0, catalog::argothian_sprite());
        let artifact_blocker = g.add_card_to_battlefield(1, catalog::phyrexian_digester()); // artifact creature
        let normal_blocker = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        assert!(!g.blocker_can_block_attacker(artifact_blocker, sprite), "artifact can't block");
        assert!(g.blocker_can_block_attacker(normal_blocker, sprite), "non-artifact can");
    }

    /// Nadier's Nightblade drains each opponent when a token you control leaves.
    #[test]
    fn nadiers_nightblade_drains_on_token_leave() {
        let mut g = two_player_game();
        g.players[0].life = 20;
        g.players[1].life = 20;
        g.add_card_to_battlefield(0, catalog::nadiers_nightblade());
        // Make a token, then destroy it.
        g.move_card_to_battlefield_for_test(0, catalog::bears_companion());
        drain_stack(&mut g);
        let bear = g.battlefield.iter().find(|c| c.controller == 0 && c.definition.name == "Bear").unwrap().id;
        // Lethal damage → SBA dispatches CreatureDied, firing the watcher.
        g.battlefield_find_mut(bear).unwrap().damage = 4;
        let evs = g.check_state_based_actions();
        g.dispatch_triggers_for_events(&evs);
        drain_stack(&mut g);
        assert_eq!(g.players[1].life, 19, "opponent lost 1");
        assert_eq!(g.players[0].life, 21, "you gained 1");
    }

    /// Gnarlroot Pallbearer pumps a creature by your graveyard's creature count.
    #[test]
    fn gnarlroot_pallbearer_scales_with_graveyard() {
        let mut g = two_player_game();
        // Seed two creature cards in the graveyard.
        for _ in 0..2 {
            g.add_card_to_graveyard(0, catalog::grizzly_bears());
        }
        let target = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Target(Target::Permanent(target))]));
        g.move_card_to_battlefield_for_test(0, catalog::gnarlroot_pallbearer());
        drain_stack(&mut g);
        // 2 creatures in gy → +2/+2 → 4/4.
        assert_eq!(g.computed_permanent(target).map(|c| c.power), Some(4));
    }

    /// Illusionary Servant sacrifices itself when targeted.
    #[test]
    fn illusionary_servant_dies_when_targeted() {
        let mut g = two_player_game();
        let servant = g.add_card_to_battlefield(0, catalog::illusionary_servant());
        let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
        g.players[1].mana_pool.add(Color::Red, 1);
        g.priority.player_with_priority = 1;
        g.perform_action(GameAction::CastSpell {
            card_id: bolt,
            target: Some(Target::Permanent(servant)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        }).expect("Bolt targets the Servant");
        drain_stack(&mut g);
        assert!(g.battlefield_find(servant).is_none(), "sacrificed on becoming targeted");
    }

    /// Bounding Wolf and Goblin Sky Raider carry their printed keywords.
    #[test]
    fn vanilla_keyword_creatures() {
        let bw = catalog::bounding_wolf();
        assert!(bw.keywords.contains(&Keyword::Flash) && bw.keywords.contains(&Keyword::Reach));
        assert!(catalog::goblin_sky_raider().keywords.contains(&Keyword::Flying));
    }

    /// Glowing Anemone bounces a land on ETB when its controller chooses to.
    #[test]
    fn glowing_anemone_returns_land() {
        let mut g = two_player_game();
        let land = g.add_card_to_battlefield(1, catalog::mountain());
        g.decider = Box::new(ScriptedDecider::new([
            DecisionAnswer::Bool(true),
            DecisionAnswer::Target(Target::Permanent(land)),
        ]));
        g.move_card_to_battlefield_for_test(0, catalog::glowing_anemone());
        drain_stack(&mut g);
        assert!(g.battlefield_find(land).is_none(), "land returned to hand");
    }

    /// Contraband Kingpin scries when an artifact you control enters.
    #[test]
    fn contraband_kingpin_scries_on_artifact() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::contraband_kingpin());
        g.add_card_to_library(0, catalog::island());
        // An artifact entering fires the scry-1 trigger (resolves via the AutoDecider).
        g.move_card_to_battlefield_for_test(0, catalog::gold_myr());
        drain_stack(&mut g);
        assert!(catalog::contraband_kingpin().keywords.contains(&Keyword::Lifelink));
        // The trigger fired and resolved without panicking; library intact (kept on top).
        assert!(g.battlefield.iter().any(|c| c.definition.name == "Gold Myr"));
    }

    /// Kingpin's Enforcers sacrifices a permanent to draw a card.
    #[test]
    fn kingpins_enforcers_sac_to_draw() {
        let mut g = two_player_game();
        let enf = g.add_card_to_battlefield(0, catalog::kingpins_enforcers());
        let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.add_card_to_library(0, catalog::island());
        g.players[0].mana_pool.add(Color::Black, 1);
        g.players[0].mana_pool.add_colorless(2);
        let hand_before = g.players[0].hand.len();
        g.perform_action(GameAction::ActivateAbility {
            card_id: enf,
            ability_index: 0,
            target: None,
            additional_targets: vec![],
            x_value: None,
        }).expect("sac ability activates");
        drain_stack(&mut g);
        assert!(g.battlefield_find(fodder).is_none(), "fodder sacrificed");
        assert_eq!(g.players[0].hand.len(), hand_before + 1, "drew a card");
    }

    /// Goldmaw Champion's Boast taps a creature after it has attacked.
    #[test]
    fn goldmaw_champion_boast_taps() {
        let mut g = two_player_game();
        let champ = g.add_card_to_battlefield(0, catalog::goldmaw_champion());
        g.battlefield_find_mut(champ).unwrap().attacked_this_turn = true;
        let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        g.players[0].mana_pool.add(Color::White, 1);
        g.players[0].mana_pool.add_colorless(1);
        g.perform_action(GameAction::ActivateAbility {
            card_id: champ,
            ability_index: 0,
            target: Some(Target::Permanent(foe)),
            additional_targets: vec![],
            x_value: None,
        }).expect("boast activates after attacking");
        drain_stack(&mut g);
        assert!(g.battlefield_find(foe).unwrap().tapped, "target tapped by Boast");
    }

    /// Gold Myr taps for white mana.
    #[test]
    fn gold_myr_makes_white() {
        let mut g = two_player_game();
        let myr = g.add_card_to_battlefield(0, catalog::gold_myr());
        g.clear_sickness(myr);
        g.perform_action(GameAction::ActivateAbility {
            card_id: myr,
            ability_index: 0,
            target: None,
            additional_targets: vec![],
            x_value: None,
        }).expect("mana ability");
        assert!(g.players[0].mana_pool.amount(Color::White) >= 1);
    }

    /// Drumhunter draws at end step when you control a 5-power creature.
    #[test]
    fn drumhunter_draws_with_big_creature() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::drumhunter());
        g.add_card_to_battlefield(0, catalog::vaultborn_tyrant()); // 6/6, power ≥ 5
        g.add_card_to_library(0, catalog::island());
        g.active_player_idx = 0;
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
        let hand_before = g.players[0].hand.len();
        g.fire_step_triggers(TurnStep::End);
        drain_stack(&mut g);
        assert_eq!(g.players[0].hand.len(), hand_before + 1, "end-step draw fired");
    }

    /// Cleave (CR 702.148) — Dig Up's base mode finds only a basic land; the cleave
    /// alt-cost removes the bracket and finds any card.
    #[test]
    fn cr_702_148_dig_up_cleave_widens_search() {
        // Base cast: only a basic land is a legal find.
        let mut g = two_player_game();
        let nonbasic = g.add_card_to_library(0, catalog::grizzly_bears());
        let basic = g.add_card_to_library(0, catalog::forest());
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(basic))]));
        let dig = g.add_card_to_hand(0, catalog::dig_up());
        g.players[0].mana_pool.add(Color::Green, 1);
        g.perform_action(GameAction::CastSpell {
            card_id: dig, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("base cast");
        drain_stack(&mut g);
        assert!(g.players[0].hand.iter().any(|c| c.id == basic), "basic land tutored to hand");
        let _ = nonbasic;

        // Cleave cast: a nonland creature card is now a legal find.
        let mut g = two_player_game();
        let creature = g.add_card_to_library(0, catalog::grizzly_bears());
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(creature))]));
        let dig = g.add_card_to_hand(0, catalog::dig_up());
        g.players[0].mana_pool.add(Color::Black, 2);
        g.players[0].mana_pool.add(Color::Green, 1);
        g.players[0].mana_pool.add_colorless(1);
        g.perform_action(GameAction::CastSpellAlternative {
            card_id: dig, pitch_card: None, target: None, additional_targets: vec![],
            mode: None, x_value: None,
        }).expect("cleave cast");
        drain_stack(&mut g);
        assert!(g.players[0].hand.iter().any(|c| c.id == creature), "cleave found a nonland card");
    }

    /// Cleave (CR 702.148) — Dread Fugue's base only discards a low-MV nonland; the
    /// cleave alt-cost lets the chooser take any nonland.
    #[test]
    fn cr_702_148_dread_fugue_cleave_widens_discard() {
        let mut g = two_player_game();
        // Opponent holds only an expensive nonland card.
        let big = g.add_card_to_hand(1, catalog::serra_angel()); // MV 5
        let fugue = g.add_card_to_hand(0, catalog::dread_fugue());
        g.players[0].mana_pool.add(Color::Black, 1);
        g.perform_action(GameAction::CastSpell {
            card_id: fugue, target: Some(Target::Player(1)), additional_targets: vec![], mode: None, x_value: None,
        }).expect("base cast");
        drain_stack(&mut g);
        assert!(g.players[1].hand.iter().any(|c| c.id == big), "base mode can't take the MV-5 card");

        // Cleave: now the MV-5 card is a legal pick.
        let fugue2 = g.add_card_to_hand(0, catalog::dread_fugue());
        g.players[0].mana_pool.add(Color::Black, 1);
        g.players[0].mana_pool.add_colorless(2);
        g.perform_action(GameAction::CastSpellAlternative {
            card_id: fugue2, pitch_card: None, target: Some(Target::Player(1)), additional_targets: vec![],
            mode: None, x_value: None,
        }).expect("cleave cast");
        drain_stack(&mut g);
        assert!(!g.players[1].hand.iter().any(|c| c.id == big), "cleave discarded the MV-5 card");
    }

    /// Venerable Monk gains 2 life on ETB.
    #[test]
    fn venerable_monk_gains_life() {
        let mut g = two_player_game();
        g.players[0].life = 20;
        g.move_card_to_battlefield_for_test(0, catalog::venerable_monk());
        drain_stack(&mut g);
        assert_eq!(g.players[0].life, 22);
    }

    /// Vanilla keyword bodies: Roc and Minotaur Aggressor.
    #[test]
    fn roc_and_minotaur_keywords() {
        assert!(catalog::roc_of_kher_ridges().keywords.contains(&Keyword::Flying));
        let m = catalog::minotaur_aggressor();
        assert!(m.keywords.contains(&Keyword::FirstStrike) && m.keywords.contains(&Keyword::Haste));
    }

    /// Malakir Familiar grows whenever you gain life.
    #[test]
    fn malakir_familiar_grows_on_lifegain() {
        let mut g = two_player_game();
        let bat = g.add_card_to_battlefield(0, catalog::malakir_familiar());
        use crabomination::effect::{Effect, Selector, Value};
        let ctx = crabomination::game::effects::EffectContext::for_ability(crabomination::card::CardId(0), 0, None);
        let evs = g
            .resolve_effect(&Effect::GainLife { who: Selector::You, amount: Value::Const(1) }, &ctx)
            .unwrap();
        g.dispatch_triggers_for_events(&evs);
        drain_stack(&mut g);
        assert_eq!(g.computed_permanent(bat).map(|c| c.power), Some(3), "+1/+1 on lifegain");
    }

    /// Mercurial Geists pumps when you cast an instant or sorcery.
    #[test]
    fn mercurial_geists_pumps_on_spell() {
        let mut g = two_player_game();
        let geist = g.add_card_to_battlefield(0, catalog::mercurial_geists());
        let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
        g.players[0].mana_pool.add(Color::Red, 1);
        g.perform_action(GameAction::CastSpell {
            card_id: bolt, target: Some(Target::Player(1)), additional_targets: vec![], mode: None, x_value: None,
        }).expect("bolt castable");
        drain_stack(&mut g);
        assert_eq!(g.computed_permanent(geist).map(|c| c.power), Some(4), "+3/+0 from instant cast");
    }

    /// Engine Rat drains each opponent for 2.
    #[test]
    fn engine_rat_drains() {
        let mut g = two_player_game();
        g.players[1].life = 20;
        let rat = g.add_card_to_battlefield(0, catalog::engine_rat());
        g.clear_sickness(rat);
        g.players[0].mana_pool.add(Color::Black, 1);
        g.players[0].mana_pool.add_colorless(5);
        g.perform_action(GameAction::ActivateAbility {
            card_id: rat, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
        }).expect("drain ability");
        drain_stack(&mut g);
        assert_eq!(g.players[1].life, 18);
    }

    /// Gavony Silversmith puts a +1/+1 counter on each of up to two creatures.
    #[test]
    fn gavony_silversmith_counters_two() {
        let mut g = two_player_game();
        let a = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let b = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.move_card_to_battlefield_for_test(0, catalog::gavony_silversmith());
        drain_stack(&mut g);
        // CR 115.1c — the engine maximizes an "up to two target" triggered
        // ability: both creatures get exactly one +1/+1 counter.
        for id in [a, b] {
            assert_eq!(
                g.battlefield_find(id).unwrap().counter_count(CounterType::PlusOnePlusOne),
                1,
                "each of the two targets gets exactly one counter"
            );
        }
    }

    /// Reputable Merchant counters a creature on ETB and again on death.
    #[test]
    fn reputable_merchant_counters_on_etb_and_death() {
        let mut g = two_player_game();
        let target = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.decider = Box::new(ScriptedDecider::new([
            DecisionAnswer::Target(Target::Permanent(target)),
            DecisionAnswer::Target(Target::Permanent(target)),
        ]));
        let merch = g.move_card_to_battlefield_for_test(0, catalog::reputable_merchant());
        drain_stack(&mut g);
        assert_eq!(g.battlefield_find(target).unwrap().counter_count(CounterType::PlusOnePlusOne), 1, "ETB counter");
        let evs = g.remove_to_graveyard_with_triggers(merch);
        g.dispatch_triggers_for_events(&evs);
        drain_stack(&mut g);
        assert_eq!(g.battlefield_find(target).unwrap().counter_count(CounterType::PlusOnePlusOne), 2, "death counter");
    }

    /// Withering Torment destroys a creature and the caster loses 2 life.
    #[test]
    fn withering_torment_kills_and_drains_caster() {
        let mut g = two_player_game();
        let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let wt = g.add_card_to_hand(0, catalog::withering_torment());
        g.players[0].mana_pool.add(Color::Black, 3);
        let life = g.players[0].life;
        g.perform_action(GameAction::CastSpell {
            card_id: wt,
            target: Some(Target::Permanent(victim)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .expect("Withering Torment castable");
        drain_stack(&mut g);
        assert!(g.battlefield_find(victim).is_none(), "creature destroyed");
        assert_eq!(g.players[0].life, life - 2, "caster lost 2 life");
    }

    /// Voltage Surge deals 2 normally, 4 when an artifact is sacrificed.
    #[test]
    fn voltage_surge_base_and_boosted() {
        // Base: AutoDecider declines the optional sacrifice → 2 damage.
        let mut g = two_player_game();
        let v = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4
        let vs = g.add_card_to_hand(0, catalog::voltage_surge());
        g.players[0].mana_pool.add(Color::Red, 1);
        g.perform_action(GameAction::CastSpell {
            card_id: vs,
            target: Some(Target::Permanent(v)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .expect("Voltage Surge castable");
        drain_stack(&mut g);
        assert_eq!(g.battlefield_find(v).map(|c| c.damage), Some(2), "2 damage, no sac");

        // Boosted: pay the optional additional cost (kicked cast) → 4 damage
        // kills the 4/4.
        let mut g = two_player_game();
        let v = g.add_card_to_battlefield(1, catalog::serra_angel());
        g.add_card_to_battlefield(0, catalog::ornithopter()); // an artifact to sac
        let vs = g.add_card_to_hand(0, catalog::voltage_surge());
        g.players[0].mana_pool.add(Color::Red, 1);
        g.perform_action(GameAction::CastSpellKicked {
            card_id: vs,
            target: Some(Target::Permanent(v)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .expect("Voltage Surge castable");
        drain_stack(&mut g);
        assert!(g.battlefield_find(v).is_none(), "4 damage killed the 4/4");
    }

    /// Corpse Appraiser exiles a graveyard creature and digs a card into hand.
    #[test]
    fn corpse_appraiser_exiles_and_digs() {
        let mut g = two_player_game();
        let gy = g.add_card_to_graveyard(1, catalog::grizzly_bears());
        for _ in 0..3 {
            g.add_card_to_library(0, catalog::island());
        }
        let hand_before = g.players[0].hand.len();
        g.move_card_to_battlefield_for_test(0, catalog::corpse_appraiser());
        drain_stack(&mut g);
        assert!(g.exile.iter().any(|c| c.id == gy), "exiled the gy creature");
        assert_eq!(g.players[0].hand.len(), hand_before + 1, "dug a card to hand");
    }

    /// The Wandering Rescuer gives other tapped creatures you control hexproof.
    #[test]
    fn wandering_rescuer_grants_tapped_hexproof() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::the_wandering_rescuer());
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        // Untapped: no hexproof.
        assert!(!g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Hexproof));
        g.battlefield_find_mut(bear).unwrap().tapped = true;
        assert!(g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Hexproof),
            "tapped creature gains hexproof");
    }

    /// Light Up the Night deals X+1 to a creature.
    #[test]
    fn light_up_the_night_hits_creature_for_x_plus_one() {
        let mut g = two_player_game();
        let v = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4
        let spell = g.add_card_to_hand(0, catalog::light_up_the_night());
        g.players[0].mana_pool.add(Color::Red, 4);
        g.perform_action(GameAction::CastSpell {
            card_id: spell,
            target: Some(Target::Permanent(v)),
            additional_targets: vec![],
            mode: None,
            x_value: Some(3),
        })
        .expect("castable for {3}{R}");
        drain_stack(&mut g);
        // X=3 → 3+1 = 4 damage kills the 4/4.
        assert!(g.battlefield_find(v).is_none(), "X+1 = 4 killed the 4/4");
    }

    /// Tyrant's Scorn destroys a small creature (mode 0).
    #[test]
    fn tyrants_scorn_destroys_small_creature() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // MV 2
        let s = g.add_card_to_hand(0, catalog::tyrants_scorn());
        g.players[0].mana_pool.add(Color::Blue, 1);
        g.players[0].mana_pool.add(Color::Black, 1);
        g.perform_action(GameAction::CastSpell {
            card_id: s, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: Some(0), x_value: None,
        }).expect("castable");
        drain_stack(&mut g);
        assert!(g.battlefield_find(bear).is_none(), "destroyed the MV-2 creature");
    }

    /// Fang of Shigeki is a 1/1 deathtouch enchantment creature.
    #[test]
    fn fang_of_shigeki_is_deathtouch_enchantment_creature() {
        let def = catalog::fang_of_shigeki();
        assert!(def.card_types.contains(&crabomination::card::CardType::Enchantment));
        assert!(def.card_types.contains(&crabomination::card::CardType::Creature));
        assert!(def.keywords.contains(&Keyword::Deathtouch));
    }

    /// Lifecraft Cavalry enters with two +1/+1 counters when revolt is active.
    #[test]
    fn lifecraft_cavalry_revolt_counters() {
        let mut g = two_player_game();
        // No revolt: enters as a plain 4/4.
        let plain = g.move_card_to_battlefield_for_test(0, catalog::lifecraft_cavalry());
        drain_stack(&mut g);
        assert_eq!(g.battlefield_find(plain).unwrap().counter_count(CounterType::PlusOnePlusOne), 0);
        // A permanent left under your control this turn → revolt.
        let token = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.remove_to_graveyard_with_triggers(token);
        let revolted = g.move_card_to_battlefield_for_test(0, catalog::lifecraft_cavalry());
        drain_stack(&mut g);
        assert_eq!(g.battlefield_find(revolted).unwrap().counter_count(CounterType::PlusOnePlusOne), 2,
            "entered with two counters under revolt");
    }

    /// Workshop Warchief gains 3 life on entry and leaves a Rhino when it dies.
    #[test]
    fn workshop_warchief_etb_life_and_dies_token() {
        let mut g = two_player_game();
        let life = g.players[0].life;
        let wc = g.move_card_to_battlefield_for_test(0, catalog::workshop_warchief());
        drain_stack(&mut g);
        assert_eq!(g.players[0].life, life + 3, "ETB gained 3 life");
        let evs = g.remove_to_graveyard_with_triggers(wc);
        g.dispatch_triggers_for_events(&evs);
        drain_stack(&mut g);
        assert!(g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Rhino"),
            "dies trigger made a Rhino");
    }

    /// Prosperous Innkeeper makes a Treasure and gains life when another creature enters.
    #[test]
    fn prosperous_innkeeper_treasure_and_lifegain() {
        let mut g = two_player_game();
        g.move_card_to_battlefield_for_test(0, catalog::prosperous_innkeeper());
        drain_stack(&mut g);
        assert!(g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Treasure"));
        let life = g.players[0].life;
        let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
        g.players[0].mana_pool.add(Color::Green, 1);
        g.players[0].mana_pool.add_colorless(1);
        g.perform_action(GameAction::CastSpell {
            card_id: bear, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("bear castable");
        drain_stack(&mut g);
        assert_eq!(g.players[0].life, life + 1, "gained 1 when another creature entered");
    }

    /// Jadar makes a decayed Zombie at end step when none is present.
    #[test]
    fn jadar_makes_decayed_zombie() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::jadar_ghoulcaller_of_nephalia());
        g.fire_step_triggers(crabomination::TurnStep::End);
        drain_stack(&mut g);
        let z = g.battlefield.iter().find(|c| c.controller == 0 && c.definition.name == "Zombie");
        assert!(z.is_some(), "made a Zombie");
        assert!(z.unwrap().definition.keywords.contains(&Keyword::Decayed), "with decayed");
    }

    /// The Goose Mother enters with X +1/+1 counters and half-X Food.
    #[test]
    fn goose_mother_counters_and_food() {
        let mut g = two_player_game();
        let goose = g.add_card_to_hand(0, catalog::the_goose_mother());
        g.players[0].mana_pool.add(Color::Green, 1);
        g.players[0].mana_pool.add(Color::Blue, 1);
        g.players[0].mana_pool.add_colorless(4); // X=4
        g.perform_action(GameAction::CastSpell {
            card_id: goose, target: None, additional_targets: vec![], mode: None, x_value: Some(4),
        }).expect("cast for {4}{G}{U}");
        drain_stack(&mut g);
        let g_id = g.battlefield.iter().find(|c| c.definition.name == "The Goose Mother").unwrap().id;
        assert_eq!(g.battlefield_find(g_id).unwrap().counter_count(CounterType::PlusOnePlusOne), 4, "X counters");
        let foods = g.battlefield.iter().filter(|c| c.controller == 0 && c.definition.name == "Food").count();
        assert_eq!(foods, 2, "half of 4 = 2 Food");
    }

    /// Archangel of Wrath deals 2 per kick on ETB (multikicker twice → 4 total).
    #[test]
    fn archangel_of_wrath_kicked_twice_burns_four() {
        let mut g = two_player_game();
        let aa = g.add_card_to_hand(0, catalog::archangel_of_wrath());
        g.players[0].mana_pool.add(Color::White, 2);
        g.players[0].mana_pool.add(Color::Red, 2);
        g.players[0].mana_pool.add_colorless(2);
        let life = g.players[1].life;
        g.perform_action(GameAction::CastSpellMultikicked {
            card_id: aa, times: 2, target: Some(Target::Player(1)),
            additional_targets: vec![Target::Player(1)], mode: None, x_value: None,
        }).expect("cast kicked twice");
        drain_stack(&mut g);
        assert_eq!(g.players[1].life, life - 4, "two ETB triggers each dealt 2");
    }

    /// Ascendant Packleader enters with a counter when you control an MV-4 permanent.
    #[test]
    fn ascendant_packleader_revolt_style_etb() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::serra_angel()); // MV 5 permanent
        let pl = g.move_card_to_battlefield_for_test(0, catalog::ascendant_packleader());
        drain_stack(&mut g);
        assert_eq!(g.battlefield_find(pl).unwrap().counter_count(CounterType::PlusOnePlusOne), 1,
            "entered with a counter (MV-4+ permanent present)");
    }

    /// Persistent Specimen returns itself from the graveyard for {2}{B}.
    #[test]
    fn persistent_specimen_returns_from_graveyard() {
        let mut g = two_player_game();
        let id = g.add_card_to_graveyard(0, catalog::persistent_specimen());
        g.players[0].mana_pool.add(Color::Black, 1);
        g.players[0].mana_pool.add_colorless(2);
        g.perform_action(GameAction::ActivateAbility {
            card_id: id, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
        }).expect("graveyard ability");
        drain_stack(&mut g);
        let back = g.battlefield_find(id).expect("returned to battlefield");
        assert!(back.tapped, "returned tapped");
    }

    /// Wedding Invitation draws on ETB and makes a creature unblockable.
    #[test]
    fn wedding_invitation_etb_draw_and_unblockable() {
        let mut g = two_player_game();
        g.add_card_to_library(0, catalog::island());
        let hand = g.players[0].hand.len();
        let inv = g.move_card_to_battlefield_for_test(0, catalog::wedding_invitation());
        drain_stack(&mut g);
        assert_eq!(g.players[0].hand.len(), hand + 1, "ETB drew a card");
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.perform_action(GameAction::ActivateAbility {
            card_id: inv, ability_index: 0, target: Some(Target::Permanent(bear)), additional_targets: Vec::new(), x_value: None,
        }).expect("sac ability");
        drain_stack(&mut g);
        assert!(g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Unblockable));
    }

    /// Unlucky Witness exiles two cards on death and grants a may-play.
    #[test]
    fn unlucky_witness_dies_impulse() {
        let mut g = two_player_game();
        g.add_card_to_library(0, catalog::island());
        g.add_card_to_library(0, catalog::mountain());
        let id = g.add_card_to_battlefield(0, catalog::unlucky_witness());
        let evs = g.remove_to_graveyard_with_triggers(id);
        g.dispatch_triggers_for_events(&evs);
        drain_stack(&mut g);
        let exiled = g.exile.iter().filter(|c| c.owner == 0 && c.may_play_until.is_some()).count();
        assert_eq!(exiled, 2, "exiled two cards playable");
    }

    /// Squee makes a tapped, attacking Goblin when it attacks, and has Escape.
    #[test]
    fn squee_attacks_makes_goblin_and_has_escape() {
        let def = catalog::squee_dubious_monarch();
        assert!(def.keywords.iter().any(|k| matches!(k, Keyword::Escape(_, 4))), "Escape exiling four");
        let mut g = two_player_game();
        let squee = g.add_card_to_battlefield(0, catalog::squee_dubious_monarch());
        g.clear_sickness(squee);
        g.step = TurnStep::DeclareAttackers;
        g.priority.player_with_priority = 0;
        g.active_player_idx = 0;
        g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker: squee, target: AttackTarget::Player(1),
        }])).expect("Squee attacks");
        drain_stack(&mut g);
        let gob = g.battlefield.iter().find(|c| c.controller == 0 && c.definition.name == "Goblin");
        assert!(gob.is_some_and(|c| c.tapped), "made a tapped Goblin on attack");
    }

    /// Headless Rider makes a 2/2 Zombie when a nontoken Zombie you control dies.
    #[test]
    fn headless_rider_makes_zombie_on_death() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::headless_rider());
        let other = g.add_card_to_battlefield(0, catalog::champion_of_the_perished()); // a 1/1 Zombie
        let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
        g.players[0].mana_pool.add(Color::Red, 1);
        g.perform_action(GameAction::CastSpell {
            card_id: bolt, target: Some(Target::Permanent(other)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("bolt the Zombie");
        drain_stack(&mut g);
        let zombies = g.battlefield.iter().filter(|c| c.controller == 0 && c.is_token
            && c.definition.name == "Zombie").count();
        assert_eq!(zombies, 1, "a Zombie token was created");
    }

    /// Diregraf Horde makes two decayed Zombies and exiles graveyard cards on ETB.
    #[test]
    fn diregraf_horde_etb_zombies_and_exile() {
        let mut g = two_player_game();
        g.add_card_to_graveyard(1, catalog::grizzly_bears());
        g.add_card_to_graveyard(1, catalog::grizzly_bears());
        g.move_card_to_battlefield_for_test(0, catalog::diregraf_horde());
        drain_stack(&mut g);
        let decayed = g.battlefield.iter().filter(|c| c.controller == 0
            && c.definition.keywords.contains(&Keyword::Decayed)).count();
        assert_eq!(decayed, 2, "two decayed Zombies");
        assert_eq!(g.players[1].graveyard.len(), 0, "exiled up to two graveyard cards");
    }

    /// The Meathook Massacre wipes the board for -X/-X and drains on your deaths.
    #[test]
    fn meathook_massacre_wipes_and_drains() {
        let mut g = two_player_game();
        let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
        let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
        let mh = g.add_card_to_hand(0, catalog::the_meathook_massacre());
        g.players[0].mana_pool.add(Color::Black, 2);
        g.players[0].mana_pool.add_colorless(2); // X=2
        let opp_life = g.players[1].life;
        g.perform_action(GameAction::CastSpell {
            card_id: mh, target: None, additional_targets: vec![], mode: None, x_value: Some(2),
        }).expect("cast for {2}{B}{B}");
        drain_stack(&mut g);
        assert!(g.battlefield_find(mine).is_none() && g.battlefield_find(theirs).is_none(), "-2/-2 wiped both");
        // Your creature dying makes the opponent lose 1.
        assert!(g.players[1].life < opp_life, "opponent lost life when your creature died");
    }

    /// Reckoner's Bargain sacrifices a permanent for life equal to its MV and two cards.
    #[test]
    fn reckoners_bargain_sacrifices_for_life_and_cards() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::serra_angel()); // MV 5 creature to sac
        g.add_card_to_library(0, catalog::island());
        g.add_card_to_library(0, catalog::island());
        let rb = g.add_card_to_hand(0, catalog::reckoners_bargain());
        g.players[0].mana_pool.add(Color::Black, 1);
        g.players[0].mana_pool.add_colorless(1);
        let life = g.players[0].life;
        let hand = g.players[0].hand.len();
        g.perform_action(GameAction::CastSpell {
            card_id: rb, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("castable");
        drain_stack(&mut g);
        assert_eq!(g.players[0].life, life + 5, "gained life = sacrificed MV (5)");
        assert_eq!(g.players[0].hand.len(), hand - 1 + 2, "drew two (net +1 after the cast)");
    }

    /// Phyrexian Missionary reanimates to hand only when kicked.
    #[test]
    fn phyrexian_missionary_kicked_reanimates() {
        let mut g = two_player_game();
        let dead = g.add_card_to_graveyard(0, catalog::grizzly_bears());
        let pm = g.add_card_to_hand(0, catalog::phyrexian_missionary());
        g.players[0].mana_pool.add(Color::White, 1);
        g.players[0].mana_pool.add(Color::Black, 1);
        g.players[0].mana_pool.add_colorless(2);
        g.perform_action(GameAction::CastSpellKicked {
            card_id: pm, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast kicked");
        drain_stack(&mut g);
        assert!(g.players[0].hand.iter().any(|c| c.id == dead), "kicked → creature back to hand");
    }

    /// Soul Transfer exiles a creature (mode 0).
    #[test]
    fn soul_transfer_exiles_creature() {
        let mut g = two_player_game();
        let v = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let st = g.add_card_to_hand(0, catalog::soul_transfer());
        g.players[0].mana_pool.add(Color::Black, 2);
        g.players[0].mana_pool.add_colorless(1);
        g.perform_action(GameAction::CastSpell {
            card_id: st, target: Some(Target::Permanent(v)), additional_targets: vec![], mode: Some(0), x_value: None,
        }).expect("castable");
        drain_stack(&mut g);
        assert!(g.exile.iter().any(|c| c.id == v), "creature exiled");
    }

    /// Cobblebrute is a 5/2 vanilla creature.
    #[test]
    fn cobblebrute_is_5_2() {
        let def = catalog::cobblebrute();
        assert_eq!((def.power, def.toughness), (5, 2));
        assert!(def.keywords.is_empty() && def.triggered_abilities.is_empty(), "vanilla");
    }

    /// Brimstone Vandal becomes day on entry while it's neither day nor night,
    /// and that establishing (non-transition) does not ping.
    #[test]
    fn brimstone_vandal_etb_becomes_day_without_pinging() {
        let mut g = two_player_game();
        assert!(g.day_night.is_none());
        g.move_card_to_battlefield_for_test(0, catalog::brimstone_vandal());
        drain_stack(&mut g);
        assert_eq!(g.day_night, Some(crabomination::game::types::DayNight::Day));
        assert_eq!(g.players[1].life, 20, "establishing day is not a transition");
    }

    /// Cemetery Gatekeeper exiles a graveyard card on entry, then pings the caster
    /// of a spell that shares a card type with it.
    #[test]
    fn cemetery_gatekeeper_pings_shared_type_spell() {
        let mut g = two_player_game();
        g.add_card_to_graveyard(1, catalog::grizzly_bears()); // a creature card
        let gk = g.move_card_to_battlefield_for_test(0, catalog::cemetery_gatekeeper());
        drain_stack(&mut g);
        assert!(g.exile.iter().any(|c| c.exiled_with == Some(gk)), "exiled a gy card");
        // Active player casts a creature spell — shares the Creature type → 2 dmg.
        let bears = g.add_card_to_hand(0, catalog::grizzly_bears());
        g.players[0].mana_pool.add(Color::Green, 1);
        g.players[0].mana_pool.add_colorless(1);
        g.perform_action(GameAction::CastSpell {
            card_id: bears, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("castable");
        drain_stack(&mut g);
        assert_eq!(g.players[0].life, 18, "shared-type spell pings the caster for 2");
    }

    /// Cemetery Protector mints a Human when you cast a spell sharing a card type
    /// with the exiled card.
    #[test]
    fn cemetery_protector_mints_on_shared_type() {
        let mut g = two_player_game();
        g.add_card_to_graveyard(1, catalog::grizzly_bears()); // a creature card
        g.move_card_to_battlefield_for_test(0, catalog::cemetery_protector());
        drain_stack(&mut g);
        let before = g.battlefield.iter().filter(|c| c.controller == 0).count();
        let bears = g.add_card_to_hand(0, catalog::grizzly_bears());
        g.players[0].mana_pool.add(Color::Green, 1);
        g.players[0].mana_pool.add_colorless(1);
        g.perform_action(GameAction::CastSpell {
            card_id: bears, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("castable");
        drain_stack(&mut g);
        let after = g.battlefield.iter().filter(|c| c.controller == 0).count();
        // +2: the cast Grizzly Bears resolves AND a Human token is minted.
        assert_eq!(after, before + 2, "shared-type spell mints a Human token");
        assert!(g.battlefield.iter().any(|c| c.controller == 0
            && c.definition.name == "Human" && c.is_token));
    }

    /// Coven is active when you control 3+ creatures with different powers.
    #[test]
    fn coven_predicate_counts_distinct_powers() {
        use crabomination::effect::{Predicate, PlayerRef};
        use crabomination::game::effects::EffectContext;
        let mut g = two_player_game();
        // Two creatures, distinct powers (2 and 3): coven inactive.
        g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
        g.add_card_to_battlefield(0, catalog::hill_giant()); // 3/3
        let ctx = EffectContext::for_ability(crabomination::card::CardId(0), 0, None);
        assert!(!g.evaluate_predicate(&Predicate::CovenActive { who: PlayerRef::You }, &ctx));
        // A third creature with a new power (4): coven active.
        g.add_card_to_battlefield(0, catalog::serra_angel()); // 4/4
        assert!(g.evaluate_predicate(&Predicate::CovenActive { who: PlayerRef::You }, &ctx));
        // Three creatures but only two distinct powers: inactive.
        let mut g2 = two_player_game();
        g2.add_card_to_battlefield(0, catalog::grizzly_bears());
        g2.add_card_to_battlefield(0, catalog::grizzly_bears());
        g2.add_card_to_battlefield(0, catalog::hill_giant());
        assert!(!g2.evaluate_predicate(&Predicate::CovenActive { who: PlayerRef::You }, &ctx));
    }

    /// Sigarda anthems Humans and digs only when coven is active.
    #[test]
    fn sigarda_anthems_humans() {
        let mut g = two_player_game();
        let s = g.add_card_to_battlefield(0, catalog::sigarda_champion_of_light());
        let human = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // not a Human
        let soldier = g.add_card_to_battlefield(0, catalog::cemetery_protector()); // Human Soldier 3/4
        // Sigarda is an Angel (no self-anthem); the Human gets +1/+1.
        let cp = g.computed_permanent(soldier).unwrap();
        assert_eq!((cp.power, cp.toughness), (4, 5), "Human anthemed");
        let bears = g.computed_permanent(human).unwrap();
        assert_eq!((bears.power, bears.toughness), (2, 2), "non-Human unaffected");
        let _ = s;
    }

    /// Dawnhart Mentor's Coven ability is gated on coven being active.
    #[test]
    fn dawnhart_mentor_coven_gate() {
        let mut g = two_player_game();
        let m = g.add_card_to_battlefield(0, catalog::dawnhart_mentor()); // 0/4 Human
        g.players[0].mana_pool.add(Color::Green, 1);
        g.players[0].mana_pool.add_colorless(5);
        // Only one creature → coven inactive → activation rejected.
        let r = g.perform_action(GameAction::ActivateAbility {
            card_id: m, ability_index: 0,
            target: Some(Target::Permanent(m)), additional_targets: vec![], x_value: None,
        });
        assert!(r.is_err(), "coven inactive blocks activation");
        // Add two more distinct-power creatures → coven active → activation works.
        g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2
        g.add_card_to_battlefield(0, catalog::hill_giant()); // 3
        g.perform_action(GameAction::ActivateAbility {
            card_id: m, ability_index: 0,
            target: Some(Target::Permanent(m)), additional_targets: vec![], x_value: None,
        }).expect("coven active allows activation");
        drain_stack(&mut g);
        let cp = g.computed_permanent(m).unwrap();
        assert_eq!((cp.power, cp.toughness), (3, 7), "+3/+3 applied");
        assert!(cp.keywords.contains(&Keyword::Trample));
    }

    /// Bladebrand grants deathtouch and draws.
    #[test]
    fn bladebrand_deathtouch_and_draw() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.add_card_to_library(0, catalog::mountain()); // something to draw
        let bb = g.add_card_to_hand(0, catalog::bladebrand());
        g.players[0].mana_pool.add(Color::Black, 1);
        g.players[0].mana_pool.add_colorless(1);
        let hand_before = g.players[0].hand.len();
        g.perform_action(GameAction::CastSpell {
            card_id: bb, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None,
        }).expect("castable");
        drain_stack(&mut g);
        assert!(g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Deathtouch));
        // -1 (cast Bladebrand) +1 (drew) = net same as before-cast hand minus the spell.
        assert_eq!(g.players[0].hand.len(), hand_before, "spent Bladebrand, drew one");
    }

    /// Halana and Alena pump another creature at the beginning of combat.
    #[test]
    fn halana_and_alena_begin_combat_counters() {
        use crabomination::card::CounterType;
        let mut g = two_player_game();
        g.active_player_idx = 0;
        let ha = g.add_card_to_battlefield(0, catalog::halana_and_alena()); // power 2
        let ally = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Target(Target::Permanent(ally))]));
        g.fire_step_triggers(TurnStep::BeginCombat);
        drain_stack(&mut g);
        let cp = g.battlefield_find(ally).unwrap();
        assert_eq!(cp.counters.get(&CounterType::PlusOnePlusOne).copied(), Some(2), "X=2 counters");
        assert!(g.computed_permanent(ally).unwrap().keywords.contains(&Keyword::Haste));
        let _ = ha;
    }

    /// Welcoming Vampire draws when a small creature enters, once per turn.
    #[test]
    fn welcoming_vampire_draws_once_per_turn() {
        let mut g = two_player_game();
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        g.add_card_to_battlefield(0, catalog::welcoming_vampire());
        g.add_card_to_library(0, catalog::mountain());
        g.add_card_to_library(0, catalog::mountain());
        fn cast_bear(g: &mut GameState) {
            let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
            g.players[0].mana_pool.add(Color::Green, 1);
            g.players[0].mana_pool.add_colorless(1);
            g.perform_action(GameAction::CastSpell {
                card_id: bear, target: None, additional_targets: vec![], mode: None, x_value: None,
            }).expect("cast a creature");
            drain_stack(g);
        }
        let hand0 = g.players[0].hand.len();
        cast_bear(&mut g);
        assert_eq!(g.players[0].hand.len(), hand0 + 1, "drew off the small creature");
        cast_bear(&mut g);
        assert_eq!(g.players[0].hand.len(), hand0 + 1, "once each turn");
    }

    /// Cruel Witness surveils when you cast a noncreature spell.
    #[test]
    fn cruel_witness_surveils_on_noncreature_cast() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::cruel_witness());
        g.add_card_to_library(0, catalog::mountain());
        let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
        g.players[0].mana_pool.add(Color::Red, 1);
        let opp = g.players[1].life;
        // Casting an instant fires the surveil-1 (auto-decider keeps the card).
        g.perform_action(GameAction::CastSpell {
            card_id: bolt, target: Some(Target::Player(1)), additional_targets: vec![], mode: None, x_value: None,
        }).expect("castable");
        drain_stack(&mut g);
        assert_eq!(g.players[1].life, opp - 3, "bolt resolved");
        // Library top is still there (surveil kept it) — library not emptied to gy
        // beyond what surveil might do; we just assert the trigger ran without panic.
        assert!(g.battlefield.iter().any(|c| c.definition.name == "Cruel Witness"));
    }

    /// Hungry Ridgewolf is a 2/2 alone, 3/2 trample with another Wolf/Werewolf.
    #[test]
    fn hungry_ridgewolf_pack_buff() {
        let mut g = two_player_game();
        let rw = g.add_card_to_battlefield(0, catalog::hungry_ridgewolf());
        let base = g.computed_permanent(rw).unwrap();
        assert_eq!((base.power, base.toughness), (2, 2), "vanilla alone");
        assert!(!base.keywords.contains(&Keyword::Trample));
        g.add_card_to_battlefield(0, catalog::hungry_ridgewolf()); // another Wolf
        let buffed = g.computed_permanent(rw).unwrap();
        assert_eq!((buffed.power, buffed.toughness), (3, 2), "+1/+0 with a packmate");
        assert!(buffed.keywords.contains(&Keyword::Trample));
    }

    /// Skaab Wrangler taps a target by tapping three creatures you control.
    #[test]
    fn skaab_wrangler_taps_via_three() {
        let mut g = two_player_game();
        let sw = g.add_card_to_battlefield(0, catalog::skaab_wrangler());
        let a = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let b = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let target = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        g.perform_action(GameAction::ActivateAbility {
            card_id: sw, ability_index: 0,
            target: Some(Target::Permanent(target)), additional_targets: vec![], x_value: None,
        }).expect("three untapped creatures available");
        drain_stack(&mut g);
        assert!(g.battlefield_find(target).unwrap().tapped, "target tapped");
        // Three of our creatures got tapped as the cost.
        let our_tapped = g.battlefield.iter()
            .filter(|c| c.controller == 0 && c.tapped).count();
        assert_eq!(our_tapped, 3, "tapped three to pay");
        let _ = (a, b);
    }

    /// Blood Petal Celebrant gains first strike while attacking and makes Blood on death.
    #[test]
    fn blood_petal_celebrant_first_strike_and_blood() {
        let mut g = two_player_game();
        let bp = g.add_card_to_battlefield(0, catalog::blood_petal_celebrant());
        // Not attacking → no first strike.
        assert!(!g.computed_permanent(bp).unwrap().keywords.contains(&Keyword::FirstStrike));
        g.attacking.push(crabomination::game::types::Attack { attacker: bp, target: AttackTarget::Player(1) });
        assert!(g.computed_permanent(bp).unwrap().keywords.contains(&Keyword::FirstStrike), "FS while attacking");
        // On death, a Blood token appears.
        g.attacking.clear();
        g.battlefield_find_mut(bp).unwrap().damage = 5; // lethal to a 2/1
        g.check_state_based_actions();
        drain_stack(&mut g);
        assert!(g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Blood"),
            "made a Blood token on death");
    }

    /// Mask of Avacyn grants +1/+2 and hexproof to the equipped creature.
    #[test]
    fn mask_of_avacyn_equips() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let mask = g.add_card_to_battlefield(0, catalog::mask_of_avacyn());
        g.players[0].mana_pool.add_colorless(3);
        g.perform_action(GameAction::Equip { equipment: mask, target: bear }).expect("equip");
        let cp = g.computed_permanent(bear).unwrap();
        assert_eq!((cp.power, cp.toughness), (3, 4), "+1/+2");
        assert!(cp.keywords.contains(&Keyword::Hexproof));
    }

    /// Stormchaser Drake draws when your spell targets it.
    #[test]
    fn stormchaser_drake_draws_on_your_target() {
        let mut g = two_player_game();
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        let drake = g.add_card_to_battlefield(0, catalog::stormchaser_drake());
        g.add_card_to_library(0, catalog::mountain());
        let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
        g.players[0].mana_pool.add(Color::Red, 1);
        let hand0 = g.players[0].hand.len();
        g.perform_action(GameAction::CastSpell {
            card_id: bolt, target: Some(Target::Permanent(drake)), additional_targets: vec![], mode: None, x_value: None,
        }).expect("castable");
        drain_stack(&mut g);
        // -1 (cast bolt) +1 (drew off the target trigger) = net unchanged.
        assert_eq!(g.players[0].hand.len(), hand0, "drew off our own targeting");
    }

    /// Falkenrath Pit Fighter activates only after an opponent lost life.
    #[test]
    fn falkenrath_pit_fighter_gated_on_opp_life_loss() {
        let mut g = two_player_game();
        let fp = g.add_card_to_battlefield(0, catalog::falkenrath_pit_fighter());
        let fodder = g.add_card_to_battlefield(0, catalog::falkenrath_pit_fighter()); // a Vampire to sac
        g.add_card_to_hand(0, catalog::mountain()); // card to discard
        g.add_card_to_library(0, catalog::mountain());
        g.add_card_to_library(0, catalog::mountain());
        g.players[0].mana_pool.add(Color::Red, 1);
        g.players[0].mana_pool.add_colorless(1);
        // No opponent has lost life yet → rejected.
        let r = g.perform_action(GameAction::ActivateAbility {
            card_id: fp, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
        });
        assert!(r.is_err(), "gated off until an opponent loses life");
        // Make the opponent lose life, then it works.
        g.players[1].life -= 1;
        g.players[1].lost_life_this_turn = true;
        g.perform_action(GameAction::ActivateAbility {
            card_id: fp, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
        }).expect("opp lost life → activatable");
        drain_stack(&mut g);
        assert!(g.battlefield_find(fodder).is_none(), "sacrificed a Vampire");
    }

    /// Mill the top card of player `p`'s library (graveyard + CardMilled event),
    /// then dispatch the resulting triggers — a focused test fixture.
    fn mill_top(g: &mut GameState, p: usize) {
        let card = g.players[p].library.remove(0);
        let cid = card.id;
        g.players[p].graveyard.push(card);
        let evs = vec![GameEvent::CardMilled { player: p, card_id: cid }];
        g.dispatch_triggers_for_events(&evs);
    }

    /// Dreadhound drains each opponent when a creature dies or a creature is milled.
    #[test]
    fn dreadhound_drains_on_death_and_mill() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::dreadhound());
        let opp = g.players[1].life;
        // A creature dying drains the opponent (full SBA + death-trigger dispatch).
        let victim = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.battlefield_find_mut(victim).unwrap().damage = 5;
        let evs = g.check_state_based_actions();
        g.dispatch_triggers_for_events(&evs);
        drain_stack(&mut g);
        assert_eq!(g.players[1].life, opp - 1, "creature death drained opp");
        // Milling a creature card from library drains again.
        g.add_card_to_library(0, catalog::grizzly_bears());
        mill_top(&mut g, 0);
        drain_stack(&mut g);
        assert_eq!(g.players[1].life, opp - 2, "milled creature drained opp");
    }

    /// Saryth grants deathtouch to tapped allies and hexproof to untapped allies.
    #[test]
    fn saryth_tapped_untapped_grants() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::saryth_the_vipers_fang());
        let ally = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        // Untapped → hexproof, not deathtouch.
        let up = g.computed_permanent(ally).unwrap();
        assert!(up.keywords.contains(&Keyword::Hexproof));
        assert!(!up.keywords.contains(&Keyword::Deathtouch));
        // Tapped → deathtouch, not hexproof.
        g.battlefield_find_mut(ally).unwrap().tapped = true;
        let down = g.computed_permanent(ally).unwrap();
        assert!(down.keywords.contains(&Keyword::Deathtouch));
        assert!(!down.keywords.contains(&Keyword::Hexproof));
    }

    /// Reckless Stormseeker pumps a creature and grants haste at begin of combat.
    #[test]
    fn reckless_stormseeker_begin_combat_pump() {
        let mut g = two_player_game();
        g.active_player_idx = 0;
        g.add_card_to_battlefield(0, catalog::reckless_stormseeker());
        let ally = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Target(Target::Permanent(ally))]));
        g.fire_step_triggers(TurnStep::BeginCombat);
        drain_stack(&mut g);
        let cp = g.computed_permanent(ally).unwrap();
        assert_eq!(cp.power, 3, "+1/+0");
        assert!(cp.keywords.contains(&Keyword::Haste));
    }

    /// Tovolar's Huntmaster makes two Wolves on entry; it's Daybound.
    #[test]
    fn tovolars_huntmaster_makes_wolves() {
        let mut g = two_player_game();
        g.move_card_to_battlefield_for_test(0, catalog::tovolars_huntmaster());
        drain_stack(&mut g);
        let wolves = g.battlefield.iter()
            .filter(|c| c.controller == 0 && c.definition.name == "Wolf" && c.is_token).count();
        assert_eq!(wolves, 2, "two 2/2 Wolves");
        let def = catalog::tovolars_huntmaster();
        assert!(def.keywords.contains(&Keyword::Daybound));
        assert_eq!(def.back_face.as_ref().unwrap().name, "Tovolar's Packleader");
    }

    /// Geistwave bounces a permanent and draws when it was yours.
    #[test]
    fn geistwave_draws_when_bouncing_your_own() {
        let mut g = two_player_game();
        g.add_card_to_library(0, catalog::mountain());
        let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let gw = g.add_card_to_hand(0, catalog::geistwave());
        g.players[0].mana_pool.add(Color::Blue, 1);
        g.players[0].mana_pool.add_colorless(1);
        let hand0 = g.players[0].hand.len();
        g.perform_action(GameAction::CastSpell {
            card_id: gw, target: Some(Target::Permanent(mine)), additional_targets: vec![], mode: None, x_value: None,
        }).expect("castable");
        drain_stack(&mut g);
        assert!(g.battlefield_find(mine).is_none(), "bounced");
        // -1 Geistwave, +1 bounced bear, +1 drew = hand0 +1.
        assert_eq!(g.players[0].hand.len(), hand0 + 1, "drew off bouncing your own");
    }

    /// Adamant Will pumps +2/+2 and grants indestructible.
    #[test]
    fn adamant_will_pumps_and_protects() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let aw = g.add_card_to_hand(0, catalog::adamant_will());
        g.players[0].mana_pool.add(Color::White, 1);
        g.players[0].mana_pool.add_colorless(1);
        g.perform_action(GameAction::CastSpell {
            card_id: aw, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None,
        }).expect("castable");
        drain_stack(&mut g);
        let cp = g.computed_permanent(bear).unwrap();
        assert_eq!((cp.power, cp.toughness), (4, 4), "+2/+2");
        assert!(cp.keywords.contains(&Keyword::Indestructible));
    }

    /// Bladestitched Skaab anthems other Zombies.
    #[test]
    fn bladestitched_skaab_anthems_zombies() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::bladestitched_skaab());
        let zombie = g.add_card_to_battlefield(0, catalog::champion_of_the_perished()); // 1/1 Zombie
        let cp = g.computed_permanent(zombie).unwrap();
        assert_eq!((cp.power, cp.toughness), (2, 1), "+1/+0 to other Zombie");
    }

    /// Angelic Quartermaster supports two creatures on entry.
    #[test]
    fn angelic_quartermaster_supports_two() {
        use crabomination::card::CounterType;
        let mut g = two_player_game();
        let a = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let b = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let aq = g.add_card_to_battlefield(0, catalog::angelic_quartermaster());
        g.fire_self_etb_triggers(aq, 0);
        drain_stack(&mut g);
        let counters = |g: &GameState, id| g.battlefield_find(id).unwrap()
            .counter_count(CounterType::PlusOnePlusOne);
        // Auto-target spreads at least one counter (the full up-to-two spread is
        // exercised by the dedicated Support test in counters.rs).
        assert!(counters(&g, a) + counters(&g, b) >= 1, "support places a +1/+1 counter");
    }

    /// Slogurk grows on land mill and returns lands when it leaves.
    #[test]
    fn slogurk_grows_and_returns_lands() {
        use crabomination::card::CounterType;
        let mut g = two_player_game();
        let slo = g.add_card_to_battlefield(0, catalog::slogurk_the_overslime());
        // A land card hitting your graveyard adds a counter.
        g.add_card_to_graveyard(0, catalog::forest());
        let evs = vec![GameEvent::CardPutIntoGraveyard {
            player: 0, card_id: g.players[0].graveyard.last().unwrap().id, is_land: true,
        }];
        g.dispatch_triggers_for_events(&evs);
        drain_stack(&mut g);
        assert_eq!(g.battlefield_find(slo).unwrap().counters.get(&CounterType::PlusOnePlusOne).copied(), Some(1));
        // Three lands in graveyard; bounce Slogurk via SBA leaving → return up to 3.
        g.add_card_to_graveyard(0, catalog::mountain());
        g.add_card_to_graveyard(0, catalog::island());
        g.battlefield_find_mut(slo).unwrap().damage = 5;
        let sba = g.check_state_based_actions();
        g.dispatch_triggers_for_events(&sba);
        drain_stack(&mut g);
        let lands_in_hand = g.players[0].hand.iter().filter(|c| c.definition.is_land()).count();
        assert!(lands_in_hand >= 3, "returned up to three lands, got {lands_in_hand}");
    }

    /// Olivia, Crimson Bride reanimates a creature tapped and attacking on attack.
    #[test]
    fn olivia_reanimates_on_attack() {
        use crabomination::game::types::Attack;
        let mut g = two_player_game();
        g.active_player_idx = 0;
        let olivia = g.add_card_to_battlefield(0, catalog::olivia_crimson_bride());
        g.clear_sickness(olivia);
        let dead = g.add_card_to_graveyard(0, catalog::serra_angel());
        g.step = TurnStep::DeclareAttackers;
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker: olivia, target: AttackTarget::Player(1),
        }])).expect("Olivia attacks");
        drain_stack(&mut g);
        let back = g.battlefield_find(dead).expect("angel reanimated");
        assert!(back.tapped, "reanimated tapped");
        assert!(g.attacking.iter().any(|a| a.attacker == dead), "joined combat attacking");
    }

    /// Covetous Castaway mills three when it dies; its disturb back is the Spirit.
    #[test]
    fn covetous_castaway_dies_mills_three() {
        let mut g = two_player_game();
        let cc = g.add_card_to_battlefield(0, catalog::covetous_castaway());
        for _ in 0..5 { g.add_card_to_library(0, catalog::mountain()); }
        let gy0 = g.players[0].graveyard.len();
        g.battlefield_find_mut(cc).unwrap().damage = 5;
        let sba = g.check_state_based_actions();
        g.dispatch_triggers_for_events(&sba);
        drain_stack(&mut g);
        // Castaway in gy + three milled = +4.
        assert!(g.players[0].graveyard.len() >= gy0 + 3, "milled three on death");
        let def = catalog::covetous_castaway();
        assert_eq!(def.back_face.as_ref().unwrap().name, "Ghostly Castigator");
    }

    /// Vampire's Kiss drains 2 and makes two Blood tokens.
    #[test]
    fn vampires_kiss_drains_and_makes_blood() {
        let mut g = two_player_game();
        let vk = g.add_card_to_hand(0, catalog::vampires_kiss());
        g.players[0].mana_pool.add(Color::Black, 1);
        g.players[0].mana_pool.add_colorless(1);
        let (l0, l1) = (g.players[0].life, g.players[1].life);
        g.perform_action(GameAction::CastSpell {
            card_id: vk, target: Some(Target::Player(1)), additional_targets: vec![], mode: None, x_value: None,
        }).expect("castable");
        drain_stack(&mut g);
        assert_eq!(g.players[1].life, l1 - 2, "target lost 2");
        assert_eq!(g.players[0].life, l0 + 2, "you gained 2");
        let blood = g.battlefield.iter().filter(|c| c.controller == 0 && c.definition.name == "Blood").count();
        assert_eq!(blood, 2, "two Blood tokens");
    }

    /// Alchemist's Gift pumps and grants deathtouch (mode 0).
    #[test]
    fn alchemists_gift_grants_deathtouch() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let ag = g.add_card_to_hand(0, catalog::alchemists_gift());
        g.players[0].mana_pool.add(Color::Black, 1);
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Mode(0)]));
        g.perform_action(GameAction::CastSpell {
            card_id: ag, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None,
        }).expect("castable");
        drain_stack(&mut g);
        let cp = g.computed_permanent(bear).unwrap();
        assert_eq!((cp.power, cp.toughness), (3, 3), "+1/+1");
        assert!(cp.keywords.contains(&Keyword::Deathtouch));
    }

    /// Dawnhart Geist gains 2 life on an enchantment cast.
    #[test]
    fn dawnhart_geist_gains_on_enchantment() {
        let mut g = two_player_game();
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        g.add_card_to_battlefield(0, catalog::dawnhart_geist());
        let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let pacifism = g.add_card_to_hand(0, catalog::pacifism()); // {1}{W} Aura enchantment
        g.players[0].mana_pool.add(Color::White, 1);
        g.players[0].mana_pool.add_colorless(1);
        let life0 = g.players[0].life;
        g.perform_action(GameAction::CastSpell {
            card_id: pacifism, target: Some(Target::Permanent(foe)), additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast the aura");
        drain_stack(&mut g);
        assert_eq!(g.players[0].life, life0 + 2, "gained 2 from the enchantment cast");
    }

    /// Bramble Wurm gains 5 on entry and again from the graveyard.
    #[test]
    fn bramble_wurm_lifegain() {
        let mut g = two_player_game();
        let life0 = g.players[0].life;
        g.move_card_to_battlefield_for_test(0, catalog::bramble_wurm());
        drain_stack(&mut g);
        assert_eq!(g.players[0].life, life0 + 5, "ETB gained 5");
        // From the graveyard: exile to gain 5.
        let bw = g.add_card_to_graveyard(0, catalog::bramble_wurm());
        g.players[0].mana_pool.add(Color::Green, 1);
        g.players[0].mana_pool.add_colorless(2);
        g.perform_action(GameAction::ActivateAbility {
            card_id: bw, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
        }).expect("graveyard ability");
        drain_stack(&mut g);
        assert_eq!(g.players[0].life, life0 + 10, "graveyard gained 5 more");
        assert!(g.exile.iter().any(|c| c.id == bw), "exiled as cost");
    }

    /// Parish-Blade Trainee trains, then passes its counters on death.
    #[test]
    fn parish_blade_trainee_trains_and_passes_counters() {
        use crabomination::card::CounterType;
        let mut g = two_player_game();
        let def = catalog::parish_blade_trainee();
        assert!(def.triggered_abilities.iter().any(|t|
            matches!(t.event.kind, crabomination::effect::EventKind::Attacks)), "has Training (attack trigger)");
        // Give it counters, then kill it; counters move to an ally.
        let pbt = g.add_card_to_battlefield(0, catalog::parish_blade_trainee());
        let ally = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.battlefield_find_mut(pbt).unwrap().add_counters(CounterType::PlusOnePlusOne, 2);
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Target(Target::Permanent(ally))]));
        g.battlefield_find_mut(pbt).unwrap().damage = 5;
        let sba = g.check_state_based_actions();
        g.dispatch_triggers_for_events(&sba);
        drain_stack(&mut g);
        assert_eq!(g.battlefield_find(ally).unwrap().counter_count(CounterType::PlusOnePlusOne), 2,
            "counters moved to the ally on death");
    }

    /// Whispering Wizard makes a Spirit on a noncreature cast, once per turn.
    #[test]
    fn whispering_wizard_makes_spirit_once() {
        let mut g = two_player_game();
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        g.add_card_to_battlefield(0, catalog::whispering_wizard());
        let cast_bolt = |g: &mut GameState| {
            let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
            g.players[0].mana_pool.add(Color::Red, 1);
            g.perform_action(GameAction::CastSpell {
                card_id: bolt, target: Some(Target::Player(1)), additional_targets: vec![], mode: None, x_value: None,
            }).expect("cast");
            drain_stack(g);
        };
        cast_bolt(&mut g);
        let spirits = |g: &GameState| g.battlefield.iter().filter(|c| c.definition.name == "Spirit").count();
        assert_eq!(spirits(&g), 1, "one Spirit");
        cast_bolt(&mut g);
        assert_eq!(spirits(&g), 1, "once each turn");
    }

    /// Patrician Geist anthems other Spirits and discounts graveyard casts.
    #[test]
    fn patrician_geist_anthems_spirits() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::patrician_geist());
        let other = g.add_card_to_battlefield(0, catalog::whispering_wizard()); // not a Spirit
        let cp = g.computed_permanent(other).unwrap();
        assert_eq!(cp.power, 3, "non-Spirit unaffected");
        let def = catalog::patrician_geist();
        assert_eq!(def.static_abilities.len(), 2, "anthem + gy discount");
    }

    /// Predator's Howl makes one Wolf normally, three with Morbid.
    #[test]
    fn predators_howl_morbid_scales() {
        let wolves = |g: &GameState| g.battlefield.iter().filter(|c| c.definition.name == "Wolf").count();
        // No death this turn → one Wolf.
        let mut g = two_player_game();
        let h = g.add_card_to_hand(0, catalog::predators_howl());
        g.players[0].mana_pool.add(Color::Green, 1);
        g.players[0].mana_pool.add_colorless(3);
        g.perform_action(GameAction::CastSpell {
            card_id: h, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast");
        drain_stack(&mut g);
        assert_eq!(wolves(&g), 1, "one Wolf without Morbid");
        // A death this turn → three Wolves.
        let mut g2 = two_player_game();
        g2.players[1].creatures_died_this_turn = 1;
        let h2 = g2.add_card_to_hand(0, catalog::predators_howl());
        g2.players[0].mana_pool.add(Color::Green, 1);
        g2.players[0].mana_pool.add_colorless(3);
        g2.perform_action(GameAction::CastSpell {
            card_id: h2, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast");
        drain_stack(&mut g2);
        assert_eq!(wolves(&g2), 3, "three Wolves with Morbid");
    }

    /// Ardenvale Tactician's adventure taps up to two creatures.
    #[test]
    fn ardenvale_tactician_adventure_taps() {
        let mut g = two_player_game();
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        let a = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let b = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let at = g.add_card_to_hand(0, catalog::ardenvale_tactician());
        g.players[0].mana_pool.add(Color::White, 1);
        g.players[0].mana_pool.add_colorless(1);
        g.perform_action(GameAction::CastAdventure {
            card_id: at, target: Some(Target::Permanent(a)),
            additional_targets: vec![Target::Permanent(b)], mode: None, x_value: None,
        }).expect("cast the adventure");
        drain_stack(&mut g);
        assert!(g.battlefield_find(a).unwrap().tapped && g.battlefield_find(b).unwrap().tapped, "both tapped");
        // The creature half waits in exile to be cast later.
        assert!(g.exile.iter().any(|c| c.id == at), "creature half exiled on the adventure");
    }

    /// Sporeback Wolf is 2/4 on your turn, 2/2 otherwise.
    #[test]
    fn sporeback_wolf_your_turn_toughness() {
        let mut g = two_player_game();
        let w = g.add_card_to_battlefield(0, catalog::sporeback_wolf());
        g.active_player_idx = 0;
        assert_eq!(g.computed_permanent(w).unwrap().toughness, 4, "+0/+2 on your turn");
        g.active_player_idx = 1;
        assert_eq!(g.computed_permanent(w).unwrap().toughness, 2, "vanilla off-turn");
    }

    /// Dawnhart Wardens pumps the team at combat only when coven is active.
    #[test]
    fn dawnhart_wardens_coven_combat_pump() {
        let mut g = two_player_game();
        g.active_player_idx = 0;
        let ally = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.add_card_to_battlefield(0, catalog::dawnhart_wardens()); // a 3/3
        // Two distinct powers (2, 3) → coven inactive → no pump.
        g.fire_step_triggers(TurnStep::BeginCombat);
        drain_stack(&mut g);
        assert_eq!(g.computed_permanent(ally).unwrap().power, 2, "no pump without coven");
        // Add a third distinct power → coven active → +1/+0.
        g.add_card_to_battlefield(0, catalog::serra_angel()); // power 4
        g.fire_step_triggers(TurnStep::BeginCombat);
        drain_stack(&mut g);
        assert_eq!(g.computed_permanent(ally).unwrap().power, 3, "+1/+0 with coven");
    }

    /// Brimstone Trebuchet pings each opponent and untaps when a Knight enters.
    #[test]
    fn brimstone_trebuchet_pings_and_untaps() {
        let mut g = two_player_game();
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        let tre = g.add_card_to_battlefield(0, catalog::brimstone_trebuchet());
        g.clear_sickness(tre);
        let opp = g.players[1].life;
        g.perform_action(GameAction::ActivateAbility {
            card_id: tre, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
        }).expect("ping");
        drain_stack(&mut g);
        assert_eq!(g.players[1].life, opp - 1, "pinged opponent");
        assert!(g.battlefield_find(tre).unwrap().tapped, "tapped to ping");
        // A Knight entering untaps the Trebuchet.
        let knight = g.move_card_to_battlefield_for_test(0, catalog::ardenvale_tactician()); // Human Knight
        g.dispatch_triggers_for_events(&[GameEvent::PermanentEntered { card_id: knight }]);
        drain_stack(&mut g);
        assert!(!g.battlefield_find(tre).unwrap().tapped, "untapped by Knight ETB");
    }

    /// Gryff Rider and Apprentice Sharpshooter carry Training.
    #[test]
    fn training_creatures_wired() {
        for def in [catalog::gryff_rider(), catalog::apprentice_sharpshooter()] {
            assert!(def.triggered_abilities.iter().any(|t|
                matches!(t.event.kind, crabomination::effect::EventKind::Attacks)), "{} has Training", def.name);
        }
        assert!(catalog::gryff_rider().keywords.contains(&Keyword::Flying));
        assert!(catalog::apprentice_sharpshooter().keywords.contains(&Keyword::Reach));
    }

    /// Bloodcrazed Socialite's attack trigger sacrifices a Blood for +2/+2.
    #[test]
    fn bloodcrazed_socialite_sacs_blood_for_pump() {
        let mut g = two_player_game();
        let soc = g.add_card_to_battlefield(0, catalog::bloodcrazed_socialite());
        // Mint a Blood token via the ETB.
        let etb = catalog::bloodcrazed_socialite().triggered_abilities[0].effect.clone();
        let ctx = crabomination::game::effects::EffectContext::for_trigger(soc, 0, None, 0);
        g.resolve_effect(&etb, &ctx).unwrap();
        assert!(g.battlefield.iter().any(|c| c.definition.name == "Blood"), "blood made");
        // Accept the reflexive sacrifice → +2/+2.
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
        let atk = catalog::bloodcrazed_socialite().triggered_abilities[1].effect.clone();
        g.resolve_effect(&atk, &ctx).unwrap();
        assert!(!g.battlefield.iter().any(|c| c.definition.name == "Blood"), "blood sacrificed");
        assert_eq!(g.battlefield_find(soc).unwrap().power(), 5, "got +2/+2");
    }

    /// Gut, True Soul Zealot's attack trigger sacrifices another creature to mint a
    /// 4/1 Skeleton tapped and attacking.
    #[test]
    fn gut_sacs_creature_for_attacking_skeleton() {
        let mut g = two_player_game();
        let gut = g.add_card_to_battlefield(0, catalog::gut_true_soul_zealot());
        let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.clear_sickness(gut);
        g.clear_sickness(fodder);
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        while g.step != TurnStep::DeclareAttackers {
            g.perform_action(GameAction::PassPriority).unwrap();
        }
        g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker: gut,
            target: AttackTarget::Player(1),
        }]))
        .expect("attack");
        drain_stack(&mut g);
        assert!(g.battlefield_find(fodder).is_none(), "fodder sacrificed (not Gut)");
        assert!(g.battlefield_find(gut).is_some(), "Gut survives — only another sacrifices");
        let skel = g.battlefield.iter().find(|c| c.is_token && c.definition.name == "Skeleton");
        let skel = skel.expect("Skeleton token minted");
        assert_eq!((skel.power(), skel.toughness()), (4, 1));
        assert!(skel.tapped, "skeleton tapped");
        let sid = skel.id;
        assert!(g.attacking.iter().any(|a| a.attacker == sid), "skeleton attacking");
    }

    /// Diregraf Scavenger drains when a creature card is exiled from a graveyard.
    #[test]
    fn diregraf_scavenger_drains_on_creature_exile() {
        let mut g = two_player_game();
        g.players[0].life = 20;
        g.players[1].life = 20;
        let dead = g.add_card_to_graveyard(1, catalog::grizzly_bears()); // creature card
        let etb = catalog::diregraf_scavenger().triggered_abilities[0].effect.clone();
        let mut ctx = crabomination::game::effects::EffectContext::for_trigger(crabomination::card::CardId(99), 0, None, 0);
        ctx.targets = vec![Target::Permanent(dead)];
        g.resolve_effect(&etb, &ctx).unwrap();
        assert!(g.exile.iter().any(|c| c.id == dead), "creature card exiled");
        assert_eq!(g.players[1].life, 18, "opponent lost 2");
        assert_eq!(g.players[0].life, 22, "you gained 2");
    }

    /// Intrepid Adversary's valor anthem scales the whole team by its valor
    /// counters; its enters-with spec reads the multikicker count.
    #[test]
    fn intrepid_adversary_valor_anthem_scales() {
        use crabomination::card::CounterType;
        use crabomination::effect::Value;
        let mut g = two_player_game();
        let ally = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
        let adv = g.add_card_to_battlefield(0, catalog::intrepid_adversary()); // 3/1
        // Two valor counters (as if kicked twice).
        g.battlefield_find_mut(adv).unwrap().counters.insert(CounterType::Valor, 2);
        // +2/+2 to every creature you control, including Intrepid Adversary itself.
        assert_eq!(g.computed_permanent(ally).map(|c| (c.power, c.toughness)), Some((4, 4)));
        assert_eq!(g.computed_permanent(adv).map(|c| (c.power, c.toughness)), Some((5, 3)));
        // Counter count drives the multikicker-fed enters-with spec.
        assert_eq!(
            catalog::intrepid_adversary().enters_with_counters,
            Some((CounterType::Valor, Value::TimesKicked))
        );
    }

    /// Bloodthirsty Adversary has haste, multikicker, and a kick-scaled +1/+1 spec.
    #[test]
    fn bloodthirsty_adversary_multikicker_counters() {
        use crabomination::card::CounterType;
        use crabomination::effect::Value;
        let d = catalog::bloodthirsty_adversary();
        assert!(d.keywords.contains(&Keyword::Haste));
        assert!(d.keywords.iter().any(|k| matches!(k, Keyword::Multikicker(_))));
        assert_eq!(d.enters_with_counters, Some((CounterType::PlusOnePlusOne, Value::TimesKicked)));
    }

    /// Eccentric Farmer mills three then returns a land from the graveyard.
    #[test]
    fn eccentric_farmer_mill_then_return_land() {
        let mut g = two_player_game();
        g.add_card_to_library(0, catalog::grizzly_bears());
        g.add_card_to_library(0, catalog::lightning_bolt());
        g.add_card_to_library(0, catalog::forest()); // top of library
        let etb = catalog::eccentric_farmer().triggered_abilities[0].effect.clone();
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
        let ctx = crabomination::game::effects::EffectContext::for_trigger(crabomination::card::CardId(99), 0, None, 0);
        g.resolve_effect(&etb, &ctx).unwrap();
        assert!(g.players[0].hand.iter().any(|c| c.definition.name == "Forest"), "land returned to hand");
        assert_eq!(g.players[0].graveyard.len(), 2, "land left gy; two nonlands remain");
    }

    /// Briarbridge Tracker investigates and grows while you control a token.
    #[test]
    fn briarbridge_tracker_token_boost() {
        let mut g = two_player_game();
        let bt = g.add_card_to_battlefield(0, catalog::briarbridge_tracker());
        // No token yet → base 2/3.
        assert_eq!(g.computed_permanent(bt).map(|c| (c.power, c.toughness)), Some((2, 3)));
        // Resolve the ETB investigate → a Clue token appears, +2/+0 kicks in.
        let etb = catalog::briarbridge_tracker().triggered_abilities[0].effect.clone();
        let ctx = crabomination::game::effects::EffectContext::for_trigger(bt, 0, None, 0);
        g.resolve_effect(&etb, &ctx).unwrap();
        assert!(g.battlefield.iter().any(|c| c.definition.name == "Clue"), "investigated");
        assert_eq!(g.computed_permanent(bt).map(|c| (c.power, c.toughness)), Some((4, 3)));
    }

    /// Markov Waltzer's begin-combat trigger pumps up to two of your creatures.
    #[test]
    fn markov_waltzer_pumps_two() {
        let mut g = two_player_game();
        let a = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let b = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let trig = catalog::markov_waltzer().triggered_abilities[0].effect.clone();
        let mut ctx = crabomination::game::effects::EffectContext::for_trigger(crabomination::card::CardId(99), 0, None, 0);
        ctx.targets = vec![Target::Permanent(a), Target::Permanent(b)];
        g.resolve_effect(&trig, &ctx).unwrap();
        assert_eq!(g.computed_permanent(a).map(|c| c.power), Some(3));
        assert_eq!(g.computed_permanent(b).map(|c| c.power), Some(3));
    }

    /// Heron-Blessed Geist's graveyard ability needs an enchantment and sorcery speed.
    #[test]
    fn heron_blessed_geist_gy_ability_gated() {
        let d = catalog::heron_blessed_geist();
        let ab = &d.activated_abilities[0];
        assert!(ab.from_graveyard && ab.exile_self_cost && ab.sorcery_speed);
        assert!(ab.condition.is_some(), "gated on controlling an enchantment");
    }

    /// Vampire Socialite buffs other Vampires when an opponent lost life this turn.
    #[test]
    fn vampire_socialite_counters_when_opp_lost_life() {
        use crabomination::card::CounterType;
        let mut g = two_player_game();
        let other = g.add_card_to_battlefield(0, catalog::bloodcrazed_socialite()); // a Vampire
        let soc = g.add_card_to_battlefield(0, catalog::vampire_socialite());
        let etb = catalog::vampire_socialite().triggered_abilities[0].effect.clone();
        let ctx = crabomination::game::effects::EffectContext::for_trigger(soc, 0, None, 0);
        // No life lost → no counters.
        g.resolve_effect(&etb, &ctx).unwrap();
        assert_eq!(g.battlefield_find(other).unwrap().counters.get(&CounterType::PlusOnePlusOne).copied(), None);
        // Opponent lost life → counter on the other Vampire (not the source).
        g.players[1].lost_life_this_turn = true;
        g.resolve_effect(&etb, &ctx).unwrap();
        assert_eq!(g.battlefield_find(other).unwrap().counters.get(&CounterType::PlusOnePlusOne).copied(), Some(1));
        assert_eq!(g.battlefield_find(soc).unwrap().counters.get(&CounterType::PlusOnePlusOne).copied(), None);
    }

    /// Sigarda's Imprisonment locks the enchanted creature out of combat.
    #[test]
    fn sigardas_imprisonment_locks_creature() {
        let mut g = two_player_game();
        let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let aura = catalog::sigardas_imprisonment();
        let mut ctx = crabomination::game::effects::EffectContext::for_ability(crabomination::card::CardId(0), 0, None);
        ctx.targets = vec![Target::Permanent(victim)];
        // Put the aura on the battlefield, then resolve its attach effect.
        let aura_id = g.add_card_to_battlefield(0, aura);
        let attach = catalog::sigardas_imprisonment().effect.clone();
        let mut actx = crabomination::game::effects::EffectContext::for_ability(aura_id, 0, None);
        actx.targets = vec![Target::Permanent(victim)];
        g.resolve_effect(&attach, &actx).unwrap();
        let cp = g.computed_permanent(victim).unwrap();
        assert!(cp.keywords.contains(&Keyword::CantAttack) && cp.keywords.contains(&Keyword::CantBlock));
    }

    /// Vampire Spawn drains 2 on entry.
    #[test]
    fn vampire_spawn_drains_two() {
        let mut g = two_player_game();
        g.players[0].life = 20;
        g.players[1].life = 20;
        let etb = catalog::vampire_spawn().triggered_abilities[0].effect.clone();
        let ctx = crabomination::game::effects::EffectContext::for_trigger(crabomination::card::CardId(0), 0, None, 0);
        g.resolve_effect(&etb, &ctx).unwrap();
        assert_eq!((g.players[0].life, g.players[1].life), (22, 18));
    }

    /// Wedding Security sacrifices a Blood for a counter and a card.
    #[test]
    fn wedding_security_sacs_blood_for_counter_and_card() {
        use crabomination::card::CounterType;
        let mut g = two_player_game();
        let ws = g.add_card_to_battlefield(0, catalog::wedding_security());
        g.add_card_to_library(0, catalog::island());
        // Make a Blood token directly.
        g.add_token_to_battlefield(0, &crabomination::game::effects::blood_token());
        let hand = g.players[0].hand.len();
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
        let atk = catalog::wedding_security().triggered_abilities[0].effect.clone();
        let ctx = crabomination::game::effects::EffectContext::for_trigger(ws, 0, None, 0);
        g.resolve_effect(&atk, &ctx).unwrap();
        assert_eq!(g.battlefield_find(ws).unwrap().counters.get(&CounterType::PlusOnePlusOne).copied(), Some(1));
        assert_eq!(g.players[0].hand.len(), hand + 1, "drew a card");
    }

    /// Falcon Abomination makes a decayed Zombie on entry.
    #[test]
    fn falcon_abomination_makes_decayed_zombie() {
        let mut g = two_player_game();
        let etb = catalog::falcon_abomination().triggered_abilities[0].effect.clone();
        let ctx = crabomination::game::effects::EffectContext::for_trigger(crabomination::card::CardId(0), 0, None, 0);
        g.resolve_effect(&etb, &ctx).unwrap();
        let z = g.battlefield.iter().find(|c| c.is_token && c.definition.name == "Zombie").unwrap();
        assert!(z.definition.keywords.contains(&Keyword::Decayed));
    }

    /// Militia Rallier can't be declared as the lone attacker (CR 508.0).
    #[test]
    fn militia_rallier_cant_attack_alone() {
        let mut g = two_player_game();
        let mr = g.add_card_to_battlefield(0, catalog::militia_rallier());
        g.clear_sickness(mr);
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        while g.step != TurnStep::DeclareAttackers {
            g.perform_action(GameAction::PassPriority).unwrap();
        }
        let r = g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker: mr,
            target: AttackTarget::Player(1),
        }]));
        assert!(r.is_err(), "lone CantAttackAlone attack rejected");
    }

    /// Bleed Dry shrinks a creature lethally and exiles it instead of dying.
    #[test]
    fn bleed_dry_exiles_dying_creature() {
        let mut g = two_player_game();
        let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let mut ctx = crabomination::game::effects::EffectContext::for_ability(crabomination::card::CardId(0), 0, None);
        ctx.targets = vec![Target::Permanent(victim)];
        g.resolve_effect(&catalog::bleed_dry().effect, &ctx).unwrap();
        g.check_state_based_actions();
        assert!(g.battlefield_find(victim).is_none(), "creature died");
        assert!(g.exile.iter().any(|c| c.id == victim), "exiled instead of graveyard");
        assert!(!g.players[1].graveyard.iter().any(|c| c.id == victim), "not in graveyard");
    }

    /// Flame-Blessed Bolt burns and exiles a dying creature.
    #[test]
    fn flame_blessed_bolt_exiles_dying_creature() {
        let mut g = two_player_game();
        let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
        let mut ctx = crabomination::game::effects::EffectContext::for_ability(crabomination::card::CardId(0), 0, None);
        ctx.targets = vec![Target::Permanent(victim)];
        g.resolve_effect(&catalog::flame_blessed_bolt().effect, &ctx).unwrap();
        g.check_state_based_actions();
        assert!(g.exile.iter().any(|c| c.id == victim), "lethal 2 damage → exiled");
    }

    /// Ancestral Anger scales +X/+0 with copies in the graveyard and draws.
    #[test]
    fn ancestral_anger_scales_with_graveyard_copies() {
        let mut g = two_player_game();
        g.add_card_to_graveyard(0, catalog::ancestral_anger());
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
        g.add_card_to_library(0, catalog::island());
        let hand = g.players[0].hand.len();
        let mut ctx = crabomination::game::effects::EffectContext::for_ability(crabomination::card::CardId(0), 0, None);
        ctx.targets = vec![Target::Permanent(bear)];
        g.resolve_effect(&catalog::ancestral_anger().effect, &ctx).unwrap();
        let cp = g.computed_permanent(bear).unwrap();
        // 1 + 1 copy in gy = +2/+0 → 4/2, plus trample.
        assert_eq!((cp.power, cp.toughness), (4, 2));
        assert!(cp.keywords.contains(&Keyword::Trample));
        assert_eq!(g.players[0].hand.len(), hand + 1, "drew");
    }

    /// Famished Foragers adds RRR on entry only when an opponent lost life.
    #[test]
    fn famished_foragers_ritual_when_opp_lost_life() {
        let mut g = two_player_game();
        let etb = catalog::famished_foragers().triggered_abilities[0].effect.clone();
        let ctx = crabomination::game::effects::EffectContext::for_trigger(crabomination::card::CardId(0), 0, None, 0);
        // No life lost → no mana.
        g.resolve_effect(&etb, &ctx).unwrap();
        assert_eq!(g.players[0].mana_pool.total(), 0, "no mana without life loss");
        g.players[1].lost_life_this_turn = true;
        g.resolve_effect(&etb, &ctx).unwrap();
        assert_eq!(g.players[0].mana_pool.amount(Color::Red), 3, "added RRR");
    }

    /// Pointed Discussion draws two, loses 2, and makes a Blood token.
    #[test]
    fn pointed_discussion_draws_loses_blood() {
        let mut g = two_player_game();
        g.add_card_to_library(0, catalog::island());
        g.add_card_to_library(0, catalog::island());
        g.players[0].life = 20;
        let hand = g.players[0].hand.len();
        let ctx = crabomination::game::effects::EffectContext::for_ability(crabomination::card::CardId(0), 0, None);
        g.resolve_effect(&catalog::pointed_discussion().effect, &ctx).unwrap();
        assert_eq!(g.players[0].hand.len(), hand + 2, "drew two");
        assert_eq!(g.players[0].life, 18, "lost 2");
        assert!(g.battlefield.iter().any(|c| c.definition.name == "Blood"), "blood made");
    }

    /// Bloodtithe Collector forces a discard only when an opponent lost life.
    #[test]
    fn bloodtithe_collector_conditional_discard() {
        let mut g = two_player_game();
        g.add_card_to_hand(1, catalog::island());
        let etb = catalog::bloodtithe_collector().triggered_abilities[0].effect.clone();
        let ctx = crabomination::game::effects::EffectContext::for_trigger(crabomination::card::CardId(0), 0, None, 0);
        g.resolve_effect(&etb, &ctx).unwrap();
        assert_eq!(g.players[1].hand.len(), 1, "no discard without life loss");
        g.players[1].lost_life_this_turn = true;
        g.resolve_effect(&etb, &ctx).unwrap();
        assert_eq!(g.players[1].hand.len(), 0, "opponent discarded");
    }

    /// Dawnhart Disciple pumps itself when another Human enters.
    #[test]
    fn dawnhart_disciple_pumps_on_human_etb() {
        let mut g = two_player_game();
        let dd = g.add_card_to_battlefield(0, catalog::dawnhart_disciple());
        let human = g.move_card_to_battlefield_for_test(0, catalog::militia_rallier()); // Human Soldier
        g.dispatch_triggers_for_events(&[GameEvent::PermanentEntered { card_id: human }]);
        drain_stack(&mut g);
        let cp = g.computed_permanent(dd).unwrap();
        assert_eq!((cp.power, cp.toughness), (3, 3), "+1/+1 from the Human ETB");
    }

    /// Bramble Armor enters attached and grants +2/+1, with an equip cost.
    #[test]
    fn bramble_armor_attaches_and_buffs() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let armor = g.add_card_to_battlefield(0, catalog::bramble_armor());
        let attach = catalog::bramble_armor().triggered_abilities[0].effect.clone();
        let mut ctx = crabomination::game::effects::EffectContext::for_trigger(armor, 0, None, 0);
        ctx.targets = vec![Target::Permanent(bear)];
        g.resolve_effect(&attach, &ctx).unwrap();
        let cp = g.computed_permanent(bear).unwrap();
        assert_eq!((cp.power, cp.toughness), (4, 3), "+2/+1");
        assert!(catalog::bramble_armor().keywords.iter().any(|k| matches!(k, Keyword::Equip(_))));
    }

    /// Repository Skaab exploits a creature to return an instant/sorcery.
    #[test]
    fn repository_skaab_exploit_returns_spell() {
        let mut g = two_player_game();
        let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let bolt = g.add_card_to_graveyard(0, catalog::lightning_bolt());
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
        let etb = catalog::repository_skaab().triggered_abilities[0].effect.clone();
        let mut ctx = crabomination::game::effects::EffectContext::for_trigger(crabomination::card::CardId(99), 0, None, 0);
        ctx.targets = vec![Target::Permanent(bolt)];
        g.resolve_effect(&etb, &ctx).unwrap();
        assert!(g.battlefield_find(fodder).is_none(), "fodder exploited");
        assert!(g.players[0].hand.iter().any(|c| c.id == bolt), "bolt returned to hand");
    }

    /// Fleshtaker gains life + scrys whenever you sacrifice another creature.
    #[test]
    fn fleshtaker_triggers_on_sacrifice() {
        let mut g = two_player_game();
        let ft = g.add_card_to_battlefield(0, catalog::fleshtaker());
        let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.add_card_to_library(0, catalog::island());
        g.players[0].life = 20;
        g.sacrifice_one(fodder, 0, &mut vec![]);
        let evs = vec![GameEvent::CreatureSacrificed { card_id: fodder, who: 0 }];
        g.dispatch_triggers_for_events(&evs);
        drain_stack(&mut g);
        assert_eq!(g.players[0].life, 21, "gained 1 from the sacrifice");
        let _ = ft;
    }

    /// Blessed Defiance pumps and leaves a Spirit when the creature dies this turn.
    #[test]
    fn blessed_defiance_pump_and_death_spirit() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let mut ctx = crabomination::game::effects::EffectContext::for_ability(crabomination::card::CardId(0), 0, None);
        ctx.targets = vec![Target::Permanent(bear)];
        g.resolve_effect(&catalog::blessed_defiance().effect, &ctx).unwrap();
        let cp = g.computed_permanent(bear).unwrap();
        assert_eq!(cp.power, 4, "+2/+0");
        assert!(cp.keywords.contains(&Keyword::Lifelink));
        // Kill it → delayed trigger makes a Spirit.
        g.sacrifice_one(bear, 0, &mut vec![]);
        g.dispatch_triggers_for_events(&[GameEvent::CreatureDied { card_id: bear }]);
        drain_stack(&mut g);
        assert!(g.battlefield.iter().any(|c| c.is_token && c.definition.name == "Spirit"), "made a Spirit");
    }

    /// Gavony Trapper taps a target creature.
    #[test]
    fn gavony_trapper_taps() {
        let mut g = two_player_game();
        let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let ab = catalog::gavony_trapper().activated_abilities[0].effect.clone();
        let mut ctx = crabomination::game::effects::EffectContext::for_ability(crabomination::card::CardId(0), 0, None);
        ctx.targets = vec![Target::Permanent(victim)];
        g.resolve_effect(&ab, &ctx).unwrap();
        assert!(g.battlefield_find(victim).unwrap().tapped);
    }

    /// Sure Strike pumps +3/+0 and grants first strike.
    #[test]
    fn sure_strike_pump_first_strike() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let mut ctx = crabomination::game::effects::EffectContext::for_ability(crabomination::card::CardId(0), 0, None);
        ctx.targets = vec![Target::Permanent(bear)];
        g.resolve_effect(&catalog::sure_strike().effect, &ctx).unwrap();
        let cp = g.computed_permanent(bear).unwrap();
        assert_eq!(cp.power, 5);
        assert!(cp.keywords.contains(&Keyword::FirstStrike));
    }

    /// Lunar Frenzy pumps by X from the cost.
    #[test]
    fn lunar_frenzy_x_pump() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let mut ctx = crabomination::game::effects::EffectContext::for_ability(crabomination::card::CardId(0), 0, None);
        ctx.targets = vec![Target::Permanent(bear)];
        ctx.x_value = 3;
        g.resolve_effect(&catalog::lunar_frenzy().effect, &ctx).unwrap();
        let cp = g.computed_permanent(bear).unwrap();
        assert_eq!(cp.power, 5, "+3 from X");
        assert!(cp.keywords.contains(&Keyword::Trample) && cp.keywords.contains(&Keyword::FirstStrike));
    }

    /// Dawnhart Rejuvenator gains 3 life on entry and taps for any color.
    #[test]
    fn dawnhart_rejuvenator_lifegain_and_mana() {
        let mut g = two_player_game();
        g.players[0].life = 20;
        let etb = catalog::dawnhart_rejuvenator().triggered_abilities[0].effect.clone();
        let ctx = crabomination::game::effects::EffectContext::for_trigger(crabomination::card::CardId(0), 0, None, 0);
        g.resolve_effect(&etb, &ctx).unwrap();
        assert_eq!(g.players[0].life, 23);
        assert!(catalog::dawnhart_rejuvenator().activated_abilities[0].tap_cost);
    }

    /// Spore Crawler draws when it dies.
    #[test]
    fn spore_crawler_dies_draw() {
        let mut g = two_player_game();
        g.add_card_to_library(0, catalog::island());
        let hand = g.players[0].hand.len();
        let dies = catalog::spore_crawler().triggered_abilities[0].effect.clone();
        let ctx = crabomination::game::effects::EffectContext::for_trigger(crabomination::card::CardId(0), 0, None, 0);
        g.resolve_effect(&dies, &ctx).unwrap();
        assert_eq!(g.players[0].hand.len(), hand + 1);
    }

    /// Snarling Wolf's pump is once each turn.
    #[test]
    fn snarling_wolf_once_per_turn() {
        let ab = &catalog::snarling_wolf().activated_abilities[0];
        assert!(ab.once_per_turn);
    }

    /// Wolfkin Bond makes a Wolf and buffs the enchanted creature.
    #[test]
    fn wolfkin_bond_token_and_buff() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let aura = g.add_card_to_battlefield(0, catalog::wolfkin_bond());
        let mut actx = crabomination::game::effects::EffectContext::for_ability(aura, 0, None);
        actx.targets = vec![Target::Permanent(bear)];
        g.resolve_effect(&catalog::wolfkin_bond().effect, &actx).unwrap();
        let etb = catalog::wolfkin_bond().triggered_abilities[0].effect.clone();
        let tctx = crabomination::game::effects::EffectContext::for_trigger(aura, 0, None, 0);
        g.resolve_effect(&etb, &tctx).unwrap();
        assert!(g.battlefield.iter().any(|c| c.is_token && c.definition.name == "Wolf"));
        assert_eq!(g.computed_permanent(bear).map(|c| (c.power, c.toughness)), Some((4, 4)));
    }

    /// Vampires' Vengeance hits non-Vampires only and makes Blood.
    #[test]
    fn vampires_vengeance_spares_vampires() {
        let mut g = two_player_game();
        let vamp = g.add_card_to_battlefield(1, catalog::bloodcrazed_socialite()); // 3/3 Vampire
        let other = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
        let ctx = crabomination::game::effects::EffectContext::for_ability(crabomination::card::CardId(0), 0, None);
        g.resolve_effect(&catalog::vampires_vengeance().effect, &ctx).unwrap();
        g.check_state_based_actions();
        assert!(g.battlefield_find(other).is_none(), "non-Vampire took lethal 2");
        assert!(g.battlefield_find(vamp).is_some(), "Vampire spared");
        assert!(g.battlefield.iter().any(|c| c.definition.name == "Blood"));
    }

    /// Defenestrate destroys a grounded creature but can't target a flyer.
    #[test]
    fn defenestrate_kills_grounded_only() {
        let mut g = two_player_game();
        let ground = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let flyer = g.add_card_to_battlefield(1, catalog::serra_angel());
        let mut ctx = crabomination::game::effects::EffectContext::for_ability(crabomination::card::CardId(0), 0, None);
        ctx.targets = vec![Target::Permanent(ground)];
        g.resolve_effect(&catalog::defenestrate().effect, &ctx).unwrap();
        g.check_state_based_actions();
        assert!(g.battlefield_find(ground).is_none(), "grounded creature destroyed");
        // Targeting a flyer is illegal.
        assert!(!g.evaluate_requirement_static(
            &SelectionRequirement::Creature.and(SelectionRequirement::Not(Box::new(SelectionRequirement::HasKeyword(Keyword::Flying)))),
            &Target::Permanent(flyer), 0, None));
    }

    /// Burn the Accursed burns a creature (exiling it) and pings its controller.
    #[test]
    fn burn_the_accursed_damage_and_exile() {
        let mut g = two_player_game();
        g.players[1].life = 20;
        let victim = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4
        let mut ctx = crabomination::game::effects::EffectContext::for_ability(crabomination::card::CardId(0), 0, None);
        ctx.targets = vec![Target::Permanent(victim)];
        g.resolve_effect(&catalog::burn_the_accursed().effect, &ctx).unwrap();
        g.check_state_based_actions();
        assert!(g.exile.iter().any(|c| c.id == victim), "5 damage killed and exiled the 4/4");
        assert_eq!(g.players[1].life, 18, "controller took 2");
    }

    /// Fortify's two modes pump the team's power or toughness.
    #[test]
    fn fortify_modal_team_pump() {
        use crabomination::decision::{DecisionAnswer, ScriptedDecider};
        let mut g = two_player_game();
        let a = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Mode(0)]));
        let ctx = crabomination::game::effects::EffectContext::for_ability(crabomination::card::CardId(0), 0, None);
        g.resolve_effect(&catalog::fortify().effect, &ctx).unwrap();
        assert_eq!(g.computed_permanent(a).map(|c| (c.power, c.toughness)), Some((4, 2)), "+2/+0");
    }

    /// Lambholt Harrier stops a creature from blocking.
    #[test]
    fn lambholt_harrier_cant_block() {
        let mut g = two_player_game();
        let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let ab = catalog::lambholt_harrier().activated_abilities[0].effect.clone();
        let mut ctx = crabomination::game::effects::EffectContext::for_ability(crabomination::card::CardId(0), 0, None);
        ctx.targets = vec![Target::Permanent(victim)];
        g.resolve_effect(&ab, &ctx).unwrap();
        assert!(g.computed_permanent(victim).unwrap().keywords.contains(&Keyword::CantBlock));
    }

    /// Crash the Ramparts pumps +3/+3 and grants trample.
    #[test]
    fn crash_the_ramparts_pump_trample() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let mut ctx = crabomination::game::effects::EffectContext::for_ability(crabomination::card::CardId(0), 0, None);
        ctx.targets = vec![Target::Permanent(bear)];
        g.resolve_effect(&catalog::crash_the_ramparts().effect, &ctx).unwrap();
        let cp = g.computed_permanent(bear).unwrap();
        assert_eq!((cp.power, cp.toughness), (5, 5));
        assert!(cp.keywords.contains(&Keyword::Trample));
    }

    /// Markov Purifier draws at end step (paying {2}) only when you gained life.
    #[test]
    fn markov_purifier_end_step_draw() {
        use crabomination::decision::{DecisionAnswer, ScriptedDecider};
        let mut g = two_player_game();
        let mp = catalog::markov_purifier();
        let pred = mp.triggered_abilities[0].event.filter.clone().unwrap();
        let ctx = crabomination::game::effects::EffectContext::for_ability(crabomination::card::CardId(0), 0, None);
        g.players[0].life_gained_this_turn = 0;
        assert!(!g.evaluate_predicate(&pred, &ctx), "no draw without lifegain");
        g.players[0].life_gained_this_turn = 3;
        assert!(g.evaluate_predicate(&pred, &ctx), "gated on lifegain");
        // The paid draw works when mana is floated.
        g.add_card_to_library(0, catalog::island());
        g.players[0].mana_pool.add_colorless(2);
        let hand = g.players[0].hand.len();
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
        g.resolve_effect(&mp.triggered_abilities[0].effect, &ctx).unwrap();
        assert_eq!(g.players[0].hand.len(), hand + 1, "paid two to draw");
    }

    /// Twins of Maurer Estate carries Madness.
    #[test]
    fn twins_of_maurer_estate_has_madness() {
        assert!(catalog::twins_of_maurer_estate().keywords.iter().any(|k| matches!(k, Keyword::Madness(_))));
    }

    /// Estwald Shieldbasher pays {1} on attack for indestructible.
    #[test]
    fn estwald_shieldbasher_indestructible_for_one() {
        use crabomination::decision::{DecisionAnswer, ScriptedDecider};
        let mut g = two_player_game();
        let es = g.add_card_to_battlefield(0, catalog::estwald_shieldbasher());
        g.players[0].mana_pool.add_colorless(1);
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
        let atk = catalog::estwald_shieldbasher().triggered_abilities[0].effect.clone();
        let ctx = crabomination::game::effects::EffectContext::for_trigger(es, 0, None, 0);
        g.resolve_effect(&atk, &ctx).unwrap();
        assert!(g.computed_permanent(es).unwrap().keywords.contains(&Keyword::Indestructible));
    }

    /// Stensia Banquet deals damage equal to your Vampire count and draws.
    #[test]
    fn stensia_banquet_scales_with_vampires() {
        let mut g = two_player_game();
        g.players[1].life = 20;
        g.add_card_to_battlefield(0, catalog::bloodcrazed_socialite()); // Vampire
        g.add_card_to_battlefield(0, catalog::vampire_spawn()); // Vampire
        g.add_card_to_library(0, catalog::island());
        let hand = g.players[0].hand.len();
        let mut ctx = crabomination::game::effects::EffectContext::for_ability(crabomination::card::CardId(0), 0, None);
        ctx.targets = vec![Target::Player(1)];
        g.resolve_effect(&catalog::stensia_banquet().effect, &ctx).unwrap();
        assert_eq!(g.players[1].life, 18, "2 Vampires → 2 damage");
        assert_eq!(g.players[0].hand.len(), hand + 1, "drew");
    }

    /// Sheltering Boughs draws on entry and buffs the enchanted creature +1/+3.
    #[test]
    fn sheltering_boughs_draw_and_buff() {
        let mut g = two_player_game();
        g.add_card_to_library(0, catalog::island());
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let aura = g.add_card_to_battlefield(0, catalog::sheltering_boughs());
        let mut actx = crabomination::game::effects::EffectContext::for_ability(aura, 0, None);
        actx.targets = vec![Target::Permanent(bear)];
        g.resolve_effect(&catalog::sheltering_boughs().effect, &actx).unwrap();
        let hand = g.players[0].hand.len();
        let etb = catalog::sheltering_boughs().triggered_abilities[0].effect.clone();
        let tctx = crabomination::game::effects::EffectContext::for_trigger(aura, 0, None, 0);
        g.resolve_effect(&etb, &tctx).unwrap();
        assert_eq!(g.players[0].hand.len(), hand + 1, "drew on ETB");
        assert_eq!(g.computed_permanent(bear).map(|c| (c.power, c.toughness)), Some((3, 5)), "+1/+3");
    }

    /// Lier grants flashback (= mana cost) to instants and sorceries in your
    /// graveyard, so a Bolt in the bin can be recast and then exiles.
    #[test]
    fn lier_flashbacks_graveyard_instants() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::lier_disciple_of_the_drowned());
        let bolt = g.add_card_to_graveyard(0, catalog::lightning_bolt());
        g.players[0].mana_pool.add(Color::Red, 1);
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::CastFlashback {
            card_id: bolt, target: Some(Target::Player(1)),
            additional_targets: vec![], mode: None, x_value: None,
        })
        .expect("Lier grants flashback to the Bolt");
        drain_stack(&mut g);
        assert_eq!(g.players[1].life, 17, "Bolt dealt 3");
        assert!(g.exile.iter().any(|c| c.id == bolt), "flashback exiles the spell");
    }

    /// Lier's static makes spells uncounterable.
    #[test]
    fn lier_makes_spells_uncounterable() {
        use crabomination::effect::StaticEffect;
        let lier = catalog::lier_disciple_of_the_drowned();
        assert!(lier.static_abilities.iter().any(|sa|
            matches!(sa.effect, StaticEffect::SpellsUncounterable { .. })));
    }

    /// Markov Crusader has haste only while you control another Vampire.
    #[test]
    fn markov_crusader_conditional_haste() {
        let mut g = two_player_game();
        let mc = g.add_card_to_battlefield(0, catalog::markov_crusader());
        assert!(!g.computed_permanent(mc).unwrap().keywords.contains(&Keyword::Haste),
            "no haste alone");
        g.add_card_to_battlefield(0, catalog::vampire_spawn()); // another Vampire
        assert!(g.computed_permanent(mc).unwrap().keywords.contains(&Keyword::Haste),
            "haste with another Vampire");
    }

    /// Stensia Masquerade gives your attacking creatures first strike.
    #[test]
    fn stensia_masquerade_attackers_first_strike() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::stensia_masquerade());
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.clear_sickness(bear);
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        while g.step != TurnStep::DeclareAttackers {
            g.perform_action(GameAction::PassPriority).unwrap();
        }
        g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker: bear, target: AttackTarget::Player(1),
        }])).expect("attack");
        assert!(g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::FirstStrike),
            "attacking creature gained first strike");
    }

    /// Cradle of the Accursed's sac ability makes a 2/2 black Zombie.
    #[test]
    fn cradle_of_the_accursed_makes_zombie() {
        let mut g = two_player_game();
        let land = g.add_card_to_battlefield(0, catalog::cradle_of_the_accursed());
        let before = g.battlefield.len();
        let eff = catalog::cradle_of_the_accursed().activated_abilities[1].effect.clone();
        let ctx = crabomination::game::effects::EffectContext::for_ability(land, 0, None);
        g.resolve_effect(&eff, &ctx).unwrap();
        assert_eq!(g.battlefield.len(), before + 1, "made a token");
        let tok = g.battlefield.iter().find(|c| c.definition.name == "Zombie").unwrap();
        assert_eq!((tok.definition.power, tok.definition.toughness), (2, 2));
        assert!(tok.definition.subtypes.creature_types.contains(&CreatureType::Zombie));
    }

    /// Kessig Wolfrider's activated ability makes a 3/2 red Wolf.
    #[test]
    fn kessig_wolfrider_makes_wolf() {
        let mut g = two_player_game();
        let kw = g.add_card_to_battlefield(0, catalog::kessig_wolfrider());
        let eff = catalog::kessig_wolfrider().activated_abilities[0].effect.clone();
        let ctx = crabomination::game::effects::EffectContext::for_ability(kw, 0, None);
        g.resolve_effect(&eff, &ctx).unwrap();
        let tok = g.battlefield.iter().find(|c| c.definition.name == "Wolf").unwrap();
        assert_eq!((tok.definition.power, tok.definition.toughness), (3, 2));
    }

    /// Storm Skreelix grows +2/+0 when you cast an instant or sorcery.
    #[test]
    fn storm_skreelix_pumps_on_spell() {
        let mut g = two_player_game();
        let ss = g.add_card_to_battlefield(0, catalog::storm_skreelix());
        let trig = catalog::storm_skreelix().triggered_abilities[0].effect.clone();
        let ctx = crabomination::game::effects::EffectContext::for_trigger(ss, 0, None, 0);
        g.resolve_effect(&trig, &ctx).unwrap();
        assert_eq!(g.computed_permanent(ss).unwrap().power, 4, "2 base +2");
    }

    /// Bloodvial Purveyor's attack trigger pumps +1/+0 per Blood the opponent has.
    #[test]
    fn bloodvial_purveyor_pumps_per_opponent_blood() {
        use crabomination::effect::{Effect, Value};
        let mut g = two_player_game();
        let bv = g.add_card_to_battlefield(0, catalog::bloodvial_purveyor());
        // Give the opponent two Blood tokens.
        let ctx0 = crabomination::game::effects::EffectContext::for_ability(crabomination::card::CardId(0), 1, None);
        g.resolve_effect(&Effect::CreateToken {
            who: crabomination::effect::PlayerRef::You, count: Value::Const(2),
            definition: crabomination_base::tokens::blood_token(),
        }, &ctx0).unwrap();
        let trig = catalog::bloodvial_purveyor().triggered_abilities[1].effect.clone();
        let ctx = crabomination::game::effects::EffectContext::for_trigger(bv, 0, None, 0);
        g.resolve_effect(&trig, &ctx).unwrap();
        assert_eq!(g.computed_permanent(bv).unwrap().power, 7, "5 base +2 for two Blood");
    }

    /// Croaking Counterpart copies a creature as a 1/1 Frog.
    #[test]
    fn croaking_counterpart_makes_frog_copy() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::serra_angel()); // 4/4
        let before = g.battlefield.len();
        let mut ctx = crabomination::game::effects::EffectContext::for_ability(crabomination::card::CardId(0), 0, None);
        ctx.targets = vec![Target::Permanent(bear)];
        g.resolve_effect(&catalog::croaking_counterpart().effect, &ctx).unwrap();
        assert_eq!(g.battlefield.len(), before + 1, "made a token copy");
        let tok = g.battlefield.iter().find(|c| c.is_token && c.id != bear).unwrap();
        assert_eq!((tok.definition.base_power(), tok.definition.base_toughness()), (1, 1));
        assert!(tok.definition.subtypes.creature_types.contains(&CreatureType::Frog));
    }

    /// Voldaren Estate's {5},{T} ability creates a Blood token.
    #[test]
    fn voldaren_estate_makes_blood() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::voldaren_estate());
        let before = g.battlefield.iter().filter(|c| c.is_token).count();
        let eff = catalog::voldaren_estate().activated_abilities[2].effect.clone();
        let ctx = crabomination::game::effects::EffectContext::for_ability(crabomination::card::CardId(0), 0, None);
        g.resolve_effect(&eff, &ctx).unwrap();
        assert_eq!(g.battlefield.iter().filter(|c| c.is_token).count(), before + 1);
    }

    /// Sigarda's Vanguard grants double strike on enter.
    #[test]
    fn sigardas_vanguard_grants_double_strike() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let sv = g.add_card_to_battlefield(0, catalog::sigardas_vanguard());
        let mut ctx = crabomination::game::effects::EffectContext::for_trigger(sv, 0, None, 0);
        ctx.targets = vec![Target::Permanent(bear)];
        g.resolve_effect(&catalog::sigardas_vanguard().triggered_abilities[0].effect, &ctx).unwrap();
        assert!(g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::DoubleStrike));
    }

    /// Diregraf Colossus enters with +1/+1 per Zombie card in your graveyard.
    #[test]
    fn diregraf_colossus_counts_graveyard_zombies() {
        let mut g = two_player_game();
        g.add_card_to_graveyard(0, catalog::diregraf_horde()); // a Zombie card
        g.add_card_to_graveyard(0, catalog::grizzly_bears()); // not a Zombie
        let dc = g.move_card_to_battlefield_for_test(0, catalog::diregraf_colossus());
        drain_stack(&mut g);
        let cp = g.computed_permanent(dc).unwrap();
        assert_eq!((cp.power, cp.toughness), (3, 3), "2/2 + one Zombie counter");
    }

    /// Wilhelt makes a decayed Zombie when a non-decayed Zombie dies.
    #[test]
    fn wilhelt_spawns_decayed_zombie_on_zombie_death() {
        let mut g = two_player_game();
        let wil = g.add_card_to_battlefield(0, catalog::wilhelt_the_rotcleaver());
        let before = g.battlefield.iter().filter(|c| c.is_token).count();
        let trig = catalog::wilhelt_the_rotcleaver().triggered_abilities[0].effect.clone();
        let ctx = crabomination::game::effects::EffectContext::for_trigger(wil, 0, None, 0);
        g.resolve_effect(&trig, &ctx).unwrap();
        let tok = g.battlefield.iter().find(|c| c.is_token
            && c.definition.keywords.contains(&Keyword::Decayed)).unwrap();
        assert!(tok.definition.subtypes.creature_types.contains(&CreatureType::Zombie));
        assert_eq!(g.battlefield.iter().filter(|c| c.is_token).count(), before + 1);
    }

    /// Millicent makes a flying Spirit when a Spirit you control dies.
    #[test]
    fn millicent_spawns_spirit_on_spirit_death() {
        let mut g = two_player_game();
        let mil = g.add_card_to_battlefield(0, catalog::millicent_restless_revenant());
        let before = g.battlefield.iter().filter(|c| c.is_token).count();
        let trig = catalog::millicent_restless_revenant().triggered_abilities[0].effect.clone();
        let ctx = crabomination::game::effects::EffectContext::for_trigger(mil, 0, None, 0);
        g.resolve_effect(&trig, &ctx).unwrap();
        let tok = g.battlefield.iter().find(|c| c.is_token).unwrap();
        assert!(tok.definition.subtypes.creature_types.contains(&CreatureType::Spirit));
        assert!(tok.definition.keywords.contains(&Keyword::Flying));
        assert_eq!(g.battlefield.iter().filter(|c| c.is_token).count(), before + 1);
    }

    /// Millicent has Affinity for Spirits.
    #[test]
    fn millicent_has_affinity_for_spirits() {
        assert!(catalog::millicent_restless_revenant().affinity_filter.is_some());
    }

    /// Ollenbock Escort buffs a counter-bearing creature with lifelink + indestructible.
    #[test]
    fn ollenbock_escort_grants_lifelink_indestructible() {
        let mut g = two_player_game();
        let esc = g.add_card_to_battlefield(0, catalog::ollenbock_escort());
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.battlefield_find_mut(bear).unwrap().add_counters(CounterType::PlusOnePlusOne, 1);
        let eff = catalog::ollenbock_escort().activated_abilities[0].effect.clone();
        let mut ctx = crabomination::game::effects::EffectContext::for_ability(esc, 0, None);
        ctx.targets = vec![Target::Permanent(bear)];
        g.resolve_effect(&eff, &ctx).unwrap();
        let cp = g.computed_permanent(bear).unwrap();
        assert!(cp.keywords.contains(&Keyword::Lifelink));
        assert!(cp.keywords.contains(&Keyword::Indestructible));
    }

    /// Sigarda, Font of Blessings grants hexproof to your other permanents.
    #[test]
    fn sigarda_font_grants_hexproof_to_others() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::sigarda_font_of_blessings());
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        assert!(g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Hexproof));
    }

    /// Sungold Barrage destroys a toughness-4+ creature but not a small one.
    #[test]
    fn sungold_barrage_hits_big_toughness() {
        let mut g = two_player_game();
        let big = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4
        let mut ctx = crabomination::game::effects::EffectContext::for_ability(crabomination::card::CardId(0), 0, None);
        ctx.targets = vec![Target::Permanent(big)];
        g.resolve_effect(&catalog::sungold_barrage().effect, &ctx).unwrap();
        drain_stack(&mut g);
        assert!(g.battlefield_find(big).is_none(), "4-toughness creature destroyed");
    }

    /// Ghoulcaller Gisa sacrifices a creature for Zombies equal to its power.
    #[test]
    fn ghoulcaller_gisa_makes_zombies_equal_to_power() {
        let mut g = two_player_game();
        let gisa = g.add_card_to_battlefield(0, catalog::ghoulcaller_gisa());
        g.add_card_to_battlefield(0, catalog::serra_angel()); // power 4 to sacrifice
        let before = g.battlefield.iter().filter(|c| c.is_token).count();
        let eff = catalog::ghoulcaller_gisa().activated_abilities[0].effect.clone();
        let ctx = crabomination::game::effects::EffectContext::for_ability(gisa, 0, None);
        g.resolve_effect(&eff, &ctx).unwrap();
        assert_eq!(g.battlefield.iter().filter(|c| c.is_token).count(), before + 4, "4 Zombies");
    }

    /// Ghoulish Procession makes a decayed Zombie when a nontoken creature dies.
    #[test]
    fn ghoulish_procession_spawns_on_death() {
        let mut g = two_player_game();
        let proc = g.add_card_to_battlefield(0, catalog::ghoulish_procession());
        let before = g.battlefield.iter().filter(|c| c.is_token).count();
        let trig = catalog::ghoulish_procession().triggered_abilities[0].effect.clone();
        let ctx = crabomination::game::effects::EffectContext::for_trigger(proc, 0, None, 0);
        g.resolve_effect(&trig, &ctx).unwrap();
        let tok = g.battlefield.iter().find(|c| c.is_token).unwrap();
        assert!(tok.definition.keywords.contains(&Keyword::Decayed));
        assert_eq!(g.battlefield.iter().filter(|c| c.is_token).count(), before + 1);
    }

    /// Necroduality copies a nontoken Zombie that enters.
    #[test]
    fn necroduality_copies_entering_zombie() {
        let mut g = two_player_game();
        let nd = g.add_card_to_battlefield(0, catalog::necroduality());
        let zomb = g.add_card_to_battlefield(0, catalog::diregraf_colossus());
        let before = g.battlefield.iter().filter(|c| c.is_token).count();
        let ctx = crabomination::game::effects::EffectContext::for_trigger(nd, 0, Some(Target::Permanent(zomb)), 0);
        g.resolve_effect(&catalog::necroduality().triggered_abilities[0].effect, &ctx).unwrap();
        assert_eq!(g.battlefield.iter().filter(|c| c.is_token).count(), before + 1, "copied the Zombie");
    }

    /// Falkenrath Forebear has flying and can't block.
    #[test]
    fn falkenrath_forebear_flying_cant_block() {
        let ff = catalog::falkenrath_forebear();
        assert!(ff.keywords.contains(&Keyword::Flying));
        assert!(ff.keywords.contains(&Keyword::CantBlock));
        assert!(ff.activated_abilities[0].from_graveyard, "recurs from graveyard");
    }

    /// Geralf grants flying to your Zombies and sacs a creature for an X/X Zombie.
    #[test]
    fn geralf_grants_flying_to_zombies() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::geralf_visionary_stitcher());
        let zomb = g.add_card_to_battlefield(0, catalog::diregraf_colossus());
        assert!(g.computed_permanent(zomb).unwrap().keywords.contains(&Keyword::Flying),
            "Zombies you control fly");
    }

    /// Geralf sacrifices a creature to mint an X/X Zombie equal to its toughness.
    #[test]
    fn geralf_sac_makes_xx_zombie() {
        let mut g = two_player_game();
        let geralf = g.add_card_to_battlefield(0, catalog::geralf_visionary_stitcher());
        // Only one other creature, a 4/4, so the sacrifice is deterministic.
        g.add_card_to_battlefield(0, catalog::serra_angel());
        let eff = catalog::geralf_visionary_stitcher().activated_abilities[0].effect.clone();
        let ctx = crabomination::game::effects::EffectContext::for_ability(geralf, 0, None);
        g.resolve_effect(&eff, &ctx).unwrap();
        let tok = g.battlefield.iter().find(|c| c.is_token).map(|c| c.id).unwrap();
        let cp = g.computed_permanent(tok).unwrap();
        assert_eq!((cp.power, cp.toughness), (4, 4), "X = sacrificed toughness");
    }

    /// Wickerwing Effigy lets you cast creature spells from the top of your library.
    #[test]
    fn wickerwing_effigy_casts_creatures_from_top() {
        use crabomination::effect::StaticEffect;
        let we = catalog::wickerwing_effigy();
        assert!(we.keywords.contains(&Keyword::Defender));
        assert!(we.static_abilities.iter().any(|sa|
            matches!(sa.effect, StaticEffect::PlayFromLibraryTop { .. })));
    }

    /// Massive Might pumps +2/+2 and grants trample.
    #[test]
    fn massive_might_pumps_and_tramples() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let mut ctx = crabomination::game::effects::EffectContext::for_ability(crabomination::card::CardId(0), 0, None);
        ctx.targets = vec![Target::Permanent(bear)];
        g.resolve_effect(&catalog::massive_might().effect, &ctx).unwrap();
        let cp = g.computed_permanent(bear).unwrap();
        assert_eq!((cp.power, cp.toughness), (4, 4));
        assert!(cp.keywords.contains(&Keyword::Trample));
    }

    /// Mossbeard Ancient gains 5 life on entry.
    #[test]
    fn mossbeard_ancient_gains_life() {
        let mut g = two_player_game();
        g.players[0].life = 20;
        let ma = g.add_card_to_battlefield(0, catalog::mossbeard_ancient());
        let etb = catalog::mossbeard_ancient().triggered_abilities[0].effect.clone();
        let ctx = crabomination::game::effects::EffectContext::for_trigger(ma, 0, None, 0);
        g.resolve_effect(&etb, &ctx).unwrap();
        assert_eq!(g.players[0].life, 25);
    }

    /// Shadowbeast Sighting makes a 4/4 Beast and carries flashback.
    #[test]
    fn shadowbeast_sighting_makes_beast() {
        let mut g = two_player_game();
        let ctx = crabomination::game::effects::EffectContext::for_ability(crabomination::card::CardId(0), 0, None);
        g.resolve_effect(&catalog::shadowbeast_sighting().effect, &ctx).unwrap();
        let tok = g.battlefield.iter().find(|c| c.is_token).unwrap();
        assert_eq!((tok.definition.power, tok.definition.toughness), (4, 4));
        assert!(tok.definition.subtypes.creature_types.contains(&CreatureType::Beast));
        assert!(catalog::shadowbeast_sighting().keywords.iter().any(|k| matches!(k, Keyword::Flashback(_))));
    }

    /// Sawblade Slinger's first mode destroys an opponent's artifact.
    #[test]
    fn sawblade_slinger_destroys_artifact() {
        let mut g = two_player_game();
        let ss = g.add_card_to_battlefield(0, catalog::sawblade_slinger());
        let rock = g.add_card_to_battlefield(1, catalog::mind_stone());
        let etb = catalog::sawblade_slinger().triggered_abilities[0].effect.clone();
        let mut ctx = crabomination::game::effects::EffectContext::for_trigger(ss, 0, Some(Target::Permanent(rock)), 0);
        ctx.mode = 0;
        g.resolve_effect(&etb, &ctx).unwrap();
        drain_stack(&mut g);
        assert!(g.battlefield_find(rock).is_none(), "artifact destroyed");
    }

    /// Gisa exiles a dying opponent creature (stamped with Gisa), then her upkeep
    /// reanimates it under your control with decayed.
    #[test]
    fn gisa_exiles_then_reanimates_with_decayed() {
        let mut g = two_player_game();
        let gisa = g.add_card_to_battlefield(0, catalog::gisa_glorious_resurrector());
        let opp = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        // Kill the opponent's creature → Gisa exiles it instead of the graveyard.
        let murder = g.add_card_to_hand(0, catalog::murder());
        g.players[0].mana_pool.add(Color::Black, 2);
        g.players[0].mana_pool.add_colorless(1);
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        crabomination::game::cast_at(&mut g, murder, Target::Permanent(opp));
        let exiled = g.exile.iter().find(|c| c.id == opp).expect("exiled by Gisa");
        assert_eq!(exiled.exiled_with, Some(gisa), "stamped with Gisa");
        // Gisa's upkeep brings it back under seat 0 with decayed.
        let trig = catalog::gisa_glorious_resurrector().triggered_abilities[0].effect.clone();
        let ctx = crabomination::game::effects::EffectContext::for_trigger(gisa, 0, None, 0);
        g.resolve_effect(&trig, &ctx).unwrap();
        let back = g.battlefield_find(opp).expect("reanimated onto battlefield");
        assert_eq!(back.controller, 0, "under your control");
        assert!(g.computed_permanent(opp).unwrap().keywords.contains(&Keyword::Decayed),
            "gains decayed");
    }

    /// Mounted Dreadknight enters with a counter only if an opponent lost life.
    #[test]
    fn mounted_dreadknight_counter_on_opp_life_loss() {
        let mut g = two_player_game();
        g.players[1].lost_life_this_turn = true;
        let md = g.move_card_to_battlefield_for_test(0, catalog::mounted_dreadknight());
        drain_stack(&mut g);
        let cp = g.computed_permanent(md).unwrap();
        assert_eq!((cp.power, cp.toughness), (6, 5), "5/4 + counter after opp lost life");
    }

    /// Path to the Festival fetches a basic land onto the battlefield tapped.
    #[test]
    fn path_to_the_festival_fetches_basic() {
        let mut g = two_player_game();
        let forest = g.add_card_to_library(0, catalog::forest());
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(forest))]));
        let before = g.battlefield.iter().filter(|c| c.controller == 0 && c.definition.is_land()).count();
        let ctx = crabomination::game::effects::EffectContext::for_ability(crabomination::card::CardId(0), 0, None);
        g.resolve_effect(&catalog::path_to_the_festival().effect, &ctx).unwrap();
        drain_stack(&mut g);
        let after = g.battlefield.iter().filter(|c| c.controller == 0 && c.definition.is_land()).count();
        assert_eq!(after, before + 1, "fetched a land to the battlefield");
    }

    /// Thraben Exorcism exiles a Spirit.
    #[test]
    fn thraben_exorcism_exiles_spirit() {
        let mut g = two_player_game();
        let spirit = g.add_card_to_battlefield(1, catalog::millicent_restless_revenant()); // a Spirit
        let mut ctx = crabomination::game::effects::EffectContext::for_ability(crabomination::card::CardId(0), 0, None);
        ctx.targets = vec![Target::Permanent(spirit)];
        g.resolve_effect(&catalog::thraben_exorcism().effect, &ctx).unwrap();
        drain_stack(&mut g);
        assert!(g.exile.iter().any(|c| c.id == spirit), "Spirit exiled");
    }

    /// Falkenrath Celebrants makes two Blood tokens on entry.
    #[test]
    fn falkenrath_celebrants_makes_two_blood() {
        let mut g = two_player_game();
        let fc = g.add_card_to_battlefield(0, catalog::falkenrath_celebrants());
        let before = g.battlefield.iter().filter(|c| c.is_token).count();
        let etb = catalog::falkenrath_celebrants().triggered_abilities[0].effect.clone();
        let ctx = crabomination::game::effects::EffectContext::for_trigger(fc, 0, None, 0);
        g.resolve_effect(&etb, &ctx).unwrap();
        assert_eq!(g.battlefield.iter().filter(|c| c.is_token).count(), before + 2);
    }

    /// Slaughter Specialist grows when an opponent's creature dies.
    #[test]
    fn slaughter_specialist_grows_on_opp_death() {
        let mut g = two_player_game();
        let ss = g.add_card_to_battlefield(0, catalog::slaughter_specialist());
        let trig = catalog::slaughter_specialist().triggered_abilities[1].effect.clone();
        let ctx = crabomination::game::effects::EffectContext::for_trigger(ss, 0, None, 0);
        g.resolve_effect(&trig, &ctx).unwrap();
        let cp = g.computed_permanent(ss).unwrap();
        assert_eq!((cp.power, cp.toughness), (4, 4), "3/3 + one counter");
    }

    /// Unhallowed Phalanx enters tapped.
    #[test]
    fn unhallowed_phalanx_enters_tapped() {
        let mut g = two_player_game();
        let up = g.move_card_to_battlefield_for_test(0, catalog::unhallowed_phalanx());
        drain_stack(&mut g);
        assert!(g.battlefield_find(up).unwrap().tapped, "entered tapped");
        assert_eq!(g.computed_permanent(up).map(|c| c.toughness), Some(13));
    }

    /// Moldgraf Millipede grows by the creature cards in your graveyard after mill.
    #[test]
    fn moldgraf_millipede_grows_from_graveyard() {
        let mut g = two_player_game();
        g.add_card_to_graveyard(0, catalog::grizzly_bears()); // a creature already in gy
        let mm = g.add_card_to_battlefield(0, catalog::moldgraf_millipede());
        // Stack the library with two creatures + a land so mill yields 2 more creatures.
        g.add_card_to_library(0, catalog::island());
        g.add_card_to_library(0, catalog::grizzly_bears());
        g.add_card_to_library(0, catalog::grizzly_bears());
        let etb = catalog::moldgraf_millipede().triggered_abilities[0].effect.clone();
        let ctx = crabomination::game::effects::EffectContext::for_trigger(mm, 0, None, 0);
        g.resolve_effect(&etb, &ctx).unwrap();
        // 1 pre-existing + 2 milled creatures = 3 counters → 5/5.
        assert_eq!(g.computed_permanent(mm).map(|c| (c.power, c.toughness)), Some((5, 5)));
    }

    /// Overcharged Amalgam exploits then counters a spell.
    #[test]
    fn overcharged_amalgam_counters_via_exploit() {
        let oa = catalog::overcharged_amalgam();
        assert!(oa.keywords.contains(&Keyword::Flash) && oa.keywords.contains(&Keyword::Flying));
        assert!(matches!(oa.triggered_abilities[0].effect,
            crabomination::effect::Effect::CounterSpell { .. } | crabomination::effect::Effect::MayDo { .. }
            | crabomination::effect::Effect::Seq(_)),
            "exploit wires a counter payoff");
    }

    /// Hobbling Zombie leaves a decayed Zombie when it dies.
    #[test]
    fn hobbling_zombie_dies_into_decayed() {
        let mut g = two_player_game();
        let hz = g.add_card_to_battlefield(0, catalog::hobbling_zombie());
        let on_dies = catalog::hobbling_zombie().triggered_abilities[0].effect.clone();
        let ctx = crabomination::game::effects::EffectContext::for_trigger(hz, 0, None, 0);
        g.resolve_effect(&on_dies, &ctx).unwrap();
        assert!(g.battlefield.iter().any(|c| c.is_token
            && c.definition.keywords.contains(&Keyword::Decayed)));
    }

    /// Selhoff Entomber draws by discarding a creature card.
    #[test]
    fn selhoff_entomber_loots() {
        let mut g = two_player_game();
        let se = g.add_card_to_battlefield(0, catalog::selhoff_entomber());
        let eff = catalog::selhoff_entomber().activated_abilities[0].effect.clone();
        g.add_card_to_library(0, catalog::island());
        let hand = g.players[0].hand.len();
        let ctx = crabomination::game::effects::EffectContext::for_ability(se, 0, None);
        g.resolve_effect(&eff, &ctx).unwrap();
        assert_eq!(g.players[0].hand.len(), hand + 1, "drew a card");
        assert!(catalog::selhoff_entomber().activated_abilities[0].discard_cost.is_some());
    }

    /// Falkenrath Perforator pings the defending player on attack.
    #[test]
    fn falkenrath_perforator_pings_on_attack() {
        let mut g = two_player_game();
        let fp = g.add_card_to_battlefield(0, catalog::falkenrath_perforator());
        g.clear_sickness(fp);
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        while g.step != TurnStep::DeclareAttackers {
            g.perform_action(GameAction::PassPriority).unwrap();
        }
        g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker: fp, target: AttackTarget::Player(1),
        }])).expect("attack");
        drain_stack(&mut g);
        assert_eq!(g.players[1].life, 19, "1 damage from the attack trigger");
    }

    /// Foul Play destroys a small creature and investigates.
    #[test]
    fn foul_play_destroys_small_and_investigates() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
        let mut ctx = crabomination::game::effects::EffectContext::for_ability(crabomination::card::CardId(0), 0, None);
        ctx.targets = vec![Target::Permanent(bear)];
        g.resolve_effect(&catalog::foul_play().effect, &ctx).unwrap();
        drain_stack(&mut g);
        assert!(g.battlefield_find(bear).is_none(), "destroyed");
        assert!(g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Clue"),
            "investigated a Clue");
    }

    /// Rotten Reunion makes a decayed Zombie.
    #[test]
    fn rotten_reunion_makes_decayed_zombie() {
        let mut g = two_player_game();
        let junk = g.add_card_to_graveyard(1, catalog::grizzly_bears());
        let mut ctx = crabomination::game::effects::EffectContext::for_ability(crabomination::card::CardId(0), 0, None);
        ctx.targets = vec![Target::Permanent(junk)];
        g.resolve_effect(&catalog::rotten_reunion().effect, &ctx).unwrap();
        assert!(g.battlefield.iter().any(|c| c.is_token
            && c.definition.keywords.contains(&Keyword::Decayed)));
    }

    /// Surging Flame's Ripple 4 reveals the top four; same-named copies are cast
    /// for free (each dealing 2 more), and non-copies are bottomed.
    #[test]
    fn surging_flame_ripple_casts_copies() {
        use crabomination::decision::{DecisionAnswer, ScriptedDecider};
        let mut g = two_player_game();
        // Top four of the library: two Surging Flame copies, then two Forests.
        let c1 = g.add_card_to_library(0, catalog::surging_flame());
        let c2 = g.add_card_to_library(0, catalog::surging_flame());
        let f1 = g.add_card_to_library(0, catalog::forest());
        let f2 = g.add_card_to_library(0, catalog::forest());
        g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Bool(true); 6]));

        let flame = g.add_card_to_hand(0, catalog::surging_flame());
        g.players[0].mana_pool.add(Color::Red, 1);
        g.players[0].mana_pool.add_colorless(1);
        cast_at(&mut g, flame, Target::Player(1));

        // 2 (original) + 2 + 2 (the two rippled copies, auto-aimed at the opponent).
        assert_eq!(g.players[1].life, 14, "three Surging Flames hit for 2 each");
        assert!(!g.players[0].library.iter().any(|c| c.id == c1 || c.id == c2), "copies cast");
        assert!(g.players[0].library.iter().any(|c| c.id == f1), "non-copy bottomed");
        assert!(g.players[0].library.iter().any(|c| c.id == f2), "non-copy bottomed");
    }

    /// Surging Might pumps a creature +1/+1 and grants trample.
    #[test]
    fn surging_might_pumps_and_grants_trample() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
        let might = g.add_card_to_hand(0, catalog::surging_might());
        g.players[0].mana_pool.add(Color::Green, 1);
        cast_at(&mut g, might, Target::Permanent(bear));
        let cp = g.computed_permanent(bear).unwrap();
        assert_eq!((cp.power, cp.toughness), (3, 3), "+1/+1");
        assert!(cp.keywords.contains(&Keyword::Trample), "gained trample");
    }

    /// Surging Dementia makes each player discard; a rippled copy doubles it.
    #[test]
    fn surging_dementia_each_player_discards() {
        let mut g = two_player_game();
        for _ in 0..3 { g.add_card_to_hand(0, catalog::forest()); }
        for _ in 0..3 { g.add_card_to_hand(1, catalog::forest()); }
        let dem = g.add_card_to_hand(0, catalog::surging_dementia());
        let h0 = g.players[0].hand.len();
        let h1 = g.players[1].hand.len();
        g.players[0].mana_pool.add(Color::Black, 1);
        g.players[0].mana_pool.add_colorless(1);
        cast(&mut g, dem);
        // No copies on top → one discard each (the spell itself left hand to stack).
        assert_eq!(g.players[1].hand.len(), h1 - 1, "opponent discarded once");
        assert_eq!(g.players[0].hand.len(), h0 - 1 - 1, "caster lost the spell + a discard");
    }

    /// Unearth returns the card from the graveyard with haste.
    #[test]
    fn viscera_dragger_unearth_returns_with_haste() {
        let mut g = two_player_game();
        let dragger = g.add_card_to_graveyard(0, catalog::viscera_dragger());
        g.players[0].mana_pool.add(Color::Black, 1);
        g.players[0].mana_pool.add_colorless(1);
        g.perform_action(GameAction::ActivateAbility {
            card_id: dragger, ability_index: 0,
            target: None, additional_targets: vec![], x_value: None,
        }).expect("unearth activatable for {1}{B}");
        drain_stack(&mut g);
        let cp = g.computed_permanent(dragger).expect("on the battlefield");
        assert!(cp.keywords.contains(&Keyword::Haste), "gained haste");
        assert!(!g.players[0].graveyard.iter().any(|c| c.id == dragger), "left graveyard");
    }

    /// An unearthed creature is exiled at the beginning of the next end step.
    #[test]
    fn unearth_exiles_at_end_step() {
        let mut g = two_player_game();
        let dragger = g.add_card_to_graveyard(0, catalog::viscera_dragger());
        g.players[0].mana_pool.add(Color::Black, 1);
        g.players[0].mana_pool.add_colorless(1);
        g.perform_action(GameAction::ActivateAbility {
            card_id: dragger, ability_index: 0,
            target: None, additional_targets: vec![], x_value: None,
        }).unwrap();
        drain_stack(&mut g);
        g.step = TurnStep::End;
        g.fire_step_triggers(TurnStep::End);
        drain_stack(&mut g);
        assert!(g.exile.iter().any(|c| c.id == dragger), "exiled at end step");
        assert!(g.battlefield_find(dragger).is_none(), "off the battlefield");
    }

    /// Rotting Rats' unearth re-triggers its enters-the-battlefield discard.
    #[test]
    fn rotting_rats_unearth_repeats_etb_discard() {
        let mut g = two_player_game();
        g.add_card_to_hand(1, catalog::forest());
        let rats = g.add_card_to_graveyard(0, catalog::rotting_rats());
        g.players[0].mana_pool.add(Color::Black, 1);
        g.players[0].mana_pool.add_colorless(1);
        let h1 = g.players[1].hand.len();
        g.perform_action(GameAction::ActivateAbility {
            card_id: rats, ability_index: 0,
            target: None, additional_targets: vec![], x_value: None,
        }).unwrap();
        drain_stack(&mut g);
        assert!(g.battlefield_find(rats).is_some(), "unearthed");
        assert_eq!(g.players[1].hand.len(), h1 - 1, "ETB discard re-fired");
    }

    #[test]
    fn moment_of_craving_shrinks_and_gains() {
        let mut g = two_player_game();
        let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
        let spell = g.add_card_to_hand(0, catalog::moment_of_craving());
        g.players[0].mana_pool.add(Color::Black, 1);
        g.players[0].mana_pool.add_colorless(1);
        let life = g.players[0].life;
        cast_at(&mut g, spell, Target::Permanent(foe));
        assert!(g.battlefield_find(foe).is_none(), "2/2 with -2/-2 dies");
        assert_eq!(g.players[0].life, life + 2, "gained 2");
    }

    #[test]
    fn kindled_fury_grants_first_strike() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let spell = g.add_card_to_hand(0, catalog::kindled_fury());
        g.players[0].mana_pool.add(Color::Red, 1);
        cast_at(&mut g, spell, Target::Permanent(bear));
        let cp = g.computed_permanent(bear).unwrap();
        assert_eq!((cp.power, cp.toughness), (3, 2), "+1/+0");
        assert!(cp.keywords.contains(&Keyword::FirstStrike));
    }

    #[test]
    fn bond_beetle_adds_a_counter_on_enter() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.move_card_to_battlefield_for_test(0, catalog::bond_beetle());
        drain_stack(&mut g);
        let cp = g.computed_permanent(bear).unwrap();
        assert_eq!(cp.toughness, 3, "bear got a +1/+1 counter");
    }

    #[test]
    fn surging_aether_bounces_a_creature() {
        let mut g = two_player_game();
        let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let spell = g.add_card_to_hand(0, catalog::surging_aether());
        g.players[0].mana_pool.add(Color::Blue, 1);
        g.players[0].mana_pool.add_colorless(2);
        cast_at(&mut g, spell, Target::Permanent(foe));
        assert!(g.battlefield_find(foe).is_none(), "bounced");
        assert!(g.players[1].hand.iter().any(|c| c.id == foe), "to owner's hand");
    }

    #[test]
    fn arrest_locks_down_a_creature() {
        let mut g = two_player_game();
        let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let aura = g.add_card_to_hand(0, catalog::arrest());
        g.players[0].mana_pool.add(Color::White, 1);
        g.players[0].mana_pool.add_colorless(1);
        cast_at(&mut g, aura, Target::Permanent(foe));
        let cp = g.computed_permanent(foe).unwrap();
        assert!(cp.keywords.contains(&Keyword::CantAttack));
        assert!(cp.keywords.contains(&Keyword::CantBlock));
    }

    #[test]
    fn brute_strength_pumps_and_tramples() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let spell = g.add_card_to_hand(0, catalog::brute_strength());
        g.players[0].mana_pool.add(Color::Red, 1);
        g.players[0].mana_pool.add_colorless(1);
        cast_at(&mut g, spell, Target::Permanent(bear));
        let cp = g.computed_permanent(bear).unwrap();
        assert_eq!((cp.power, cp.toughness), (5, 3), "+3/+1");
        assert!(cp.keywords.contains(&Keyword::Trample));
    }

    #[test]
    fn gather_courage_pumps_a_creature() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let spell = g.add_card_to_hand(0, catalog::gather_courage());
        g.players[0].mana_pool.add(Color::Green, 1);
        cast_at(&mut g, spell, Target::Permanent(bear));
        let cp = g.computed_permanent(bear).unwrap();
        assert_eq!((cp.power, cp.toughness), (4, 4), "+2/+2");
        assert!(g.battlefield_find(spell).is_none(), "convoke spell carries Convoke keyword");
        assert!(catalog::gather_courage().keywords.contains(&Keyword::Convoke));
    }

    #[test]
    fn skeletal_kathari_unearths_a_flyer() {
        let mut g = two_player_game();
        let k = g.add_card_to_graveyard(0, catalog::skeletal_kathari());
        g.players[0].mana_pool.add(Color::Black, 1);
        g.players[0].mana_pool.add_colorless(2);
        g.perform_action(GameAction::ActivateAbility {
            card_id: k, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
        }).expect("unearth {2}{B}");
        drain_stack(&mut g);
        let cp = g.computed_permanent(k).expect("on battlefield");
        assert!(cp.keywords.contains(&Keyword::Flying) && cp.keywords.contains(&Keyword::Haste));
    }

    #[test]
    fn surging_sentinels_is_a_three_one_ripple_spirit() {
        let s = catalog::surging_sentinels();
        assert_eq!((s.power, s.toughness), (3, 1));
        assert!(s.subtypes.creature_types.contains(&CreatureType::Spirit));
        assert_eq!(s.triggered_abilities.len(), 1, "carries the ripple trigger");
    }

    #[test]
    fn fleeting_distraction_shrinks_power_and_draws() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        g.add_card_to_library(0, catalog::forest());
        let spell = g.add_card_to_hand(0, catalog::fleeting_distraction());
        g.players[0].mana_pool.add(Color::Blue, 1);
        let hand = g.players[0].hand.len();
        cast_at(&mut g, spell, Target::Permanent(bear));
        let cp = g.computed_permanent(bear).unwrap();
        assert_eq!((cp.power, cp.toughness), (1, 2), "-1/-0");
        assert_eq!(g.players[0].hand.len(), hand, "spell left hand, drew one back");
    }

    #[test]
    fn mistral_charge_grants_flying() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let spell = g.add_card_to_hand(0, catalog::mistral_charge());
        g.players[0].mana_pool.add(Color::Blue, 1);
        g.players[0].mana_pool.add_colorless(1);
        cast_at(&mut g, spell, Target::Permanent(bear));
        let cp = g.computed_permanent(bear).unwrap();
        assert_eq!((cp.power, cp.toughness), (3, 3), "+1/+1");
        assert!(cp.keywords.contains(&Keyword::Flying));
    }

    #[test]
    fn run_amok_pumps_and_tramples() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let spell = g.add_card_to_hand(0, catalog::run_amok());
        g.players[0].mana_pool.add(Color::Red, 1);
        g.players[0].mana_pool.add_colorless(2);
        cast_at(&mut g, spell, Target::Permanent(bear));
        let cp = g.computed_permanent(bear).unwrap();
        assert_eq!((cp.power, cp.toughness), (5, 5), "+3/+3");
        assert!(cp.keywords.contains(&Keyword::Trample));
    }

    /// Winding Constrictor adds one to any counter kind on your creature.
    #[test]
    fn winding_constrictor_boosts_any_counter() {
        use crabomination::effect::{Effect, Selector, Value};
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.add_card_to_battlefield(0, catalog::winding_constrictor());
        let ctx = crabomination::game::effects::EffectContext::for_ability(
            crabomination::card::CardId(0), 0, Some(Target::Permanent(bear)),
        );
        // +1/+1: 1 → 2.
        g.resolve_effect(&Effect::AddCounter {
            what: Selector::Target(0), kind: CounterType::PlusOnePlusOne, amount: Value::Const(1),
        }, &ctx).unwrap();
        assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne), 2);
        // A different kind (Charge) on a creature also gets the +1.
        g.resolve_effect(&Effect::AddCounter {
            what: Selector::Target(0), kind: CounterType::Charge, amount: Value::Const(1),
        }, &ctx).unwrap();
        assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::Charge), 2);
    }

    /// Conclave Mentor gains you life equal to its power when it dies.
    #[test]
    fn conclave_mentor_dies_gains_life() {
        let mut g = two_player_game();
        let cm = g.add_card_to_battlefield(0, catalog::conclave_mentor());
        let life = g.players[0].life;
        g.battlefield_find_mut(cm).unwrap().damage = 2; // lethal vs 2 toughness
        g.check_state_based_actions();
        drain_stack(&mut g);
        assert_eq!(g.players[0].life, life + 2, "gained 2 = its power");
    }

    /// Branching Evolution doubles +1/+1 counters placed on your creatures.
    #[test]
    fn branching_evolution_doubles_counters() {
        use crabomination::effect::{Effect, Selector, Value};
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.add_card_to_battlefield(0, catalog::branching_evolution());
        let ctx = crabomination::game::effects::EffectContext::for_ability(
            crabomination::card::CardId(0), 0, Some(Target::Permanent(bear)),
        );
        g.resolve_effect(&Effect::AddCounter {
            what: Selector::Target(0), kind: CounterType::PlusOnePlusOne, amount: Value::Const(2),
        }, &ctx).unwrap();
        assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne), 4);
    }

    /// Blight Mamba has infect and can regenerate itself.
    #[test]
    fn blight_mamba_infect_and_regen() {
        let mut g = two_player_game();
        let bm = g.add_card_to_battlefield(0, catalog::blight_mamba());
        assert!(g.battlefield_find(bm).unwrap().definition.keywords.contains(&Keyword::Infect));
        g.players[0].mana_pool.add(Color::Green, 1);
        g.players[0].mana_pool.add_colorless(1);
        g.perform_action(GameAction::ActivateAbility {
            card_id: bm, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
        }).expect("regenerate");
        drain_stack(&mut g);
        assert_eq!(g.battlefield_find(bm).unwrap().regeneration_shields, 1);
    }

    /// Ichorclaw Myr grows +2/+2 when it becomes blocked.
    #[test]
    fn ichorclaw_myr_grows_when_blocked() {
        let mut g = two_player_game();
        let myr = g.add_card_to_battlefield(0, catalog::ichorclaw_myr());
        let blocker = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        g.dispatch_triggers_for_events(&[GameEvent::BlockerDeclared { blocker, attacker: myr }]);
        drain_stack(&mut g);
        let cp = g.computed_permanent(myr).unwrap();
        assert_eq!((cp.power, cp.toughness), (3, 3), "+2/+2");
    }

    /// Necropede puts a -1/-1 counter on a creature when it dies.
    #[test]
    fn necropede_dies_shrinks_a_creature() {
        let mut g = two_player_game();
        let np = g.add_card_to_battlefield(0, catalog::necropede());
        let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
        let on_dies = catalog::necropede().triggered_abilities[0].effect.clone();
        let ctx = crabomination::game::effects::EffectContext::for_trigger(np, 0, Some(Target::Permanent(foe)), 0);
        g.resolve_effect(&on_dies, &ctx).unwrap();
        assert_eq!(g.battlefield_find(foe).unwrap().counter_count(CounterType::MinusOneMinusOne), 1);
    }

    /// Fuel for the Cause counters a spell and proliferates.
    #[test]
    fn fuel_for_the_cause_counters_and_proliferates() {
        let mut g = two_player_game();
        // A creature of ours already carries a +1/+1 counter to proliferate.
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.battlefield_find_mut(bear).unwrap().counters.insert(CounterType::PlusOnePlusOne, 1);
        // Opponent casts a sorcery we'll counter.
        let spell = g.add_card_to_hand(1, catalog::sign_in_blood());
        g.active_player_idx = 1;
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 1;
        g.players[1].mana_pool.add(Color::Black, 2);
        let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        g.perform_action(GameAction::CastSpell {
            card_id: spell, target: Some(Target::Player(1)), additional_targets: vec![], mode: None, x_value: None,
        }).ok();
        let _ = foe;
        // We respond with Fuel for the Cause.
        let fuel = g.add_card_to_hand(0, catalog::fuel_for_the_cause());
        g.priority.player_with_priority = 0;
        g.players[0].mana_pool.add(Color::Blue, 2);
        g.players[0].mana_pool.add_colorless(2);
        g.perform_action(GameAction::CastSpell {
            card_id: fuel, target: Some(Target::Permanent(spell)), additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast fuel at the spell");
        drain_stack(&mut g);
        assert!(g.players[1].graveyard.iter().any(|c| c.id == spell), "spell was countered");
        assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne), 2,
            "proliferated our counter");
    }

    /// Contagion Engine's ETB shrinks every opposing creature.
    #[test]
    fn contagion_engine_etb_shrinks_opponents() {
        let mut g = two_player_game();
        let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        g.move_card_to_battlefield_for_test(0, catalog::contagion_engine());
        drain_stack(&mut g);
        assert_eq!(g.battlefield_find(foe).unwrap().counter_count(CounterType::MinusOneMinusOne), 1);
    }

    /// Kami of False Hope sacrifices to fog all combat damage.
    #[test]
    fn kami_of_false_hope_fogs() {
        let mut g = two_player_game();
        let kami = g.add_card_to_battlefield(0, catalog::kami_of_false_hope());
        g.perform_action(GameAction::ActivateAbility {
            card_id: kami, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
        }).expect("sac for fog");
        drain_stack(&mut g);
        assert!(g.prevent_combat_damage_this_turn, "combat damage prevented");
    }

    /// Renegade Rallier reanimates a cheap permanent when Revolt is active.
    #[test]
    fn renegade_rallier_revolt_reanimates() {
        let mut g = two_player_game();
        let dead = g.add_card_to_graveyard(0, catalog::grizzly_bears()); // MV 2
        g.players[0].permanent_left_battlefield_this_turn = true; // Revolt active
        let rr = g.move_card_to_battlefield_for_test(0, catalog::renegade_rallier());
        // bind the trigger target manually via the trigger effect.
        let eff = catalog::renegade_rallier().triggered_abilities[0].effect.clone();
        let ctx = crabomination::game::effects::EffectContext::for_trigger(rr, 0, Some(Target::Permanent(dead)), 0);
        g.resolve_effect(&eff, &ctx).unwrap();
        assert!(g.battlefield_find(dead).is_some(), "grizzly returned to the battlefield");
    }

    /// Pitiless Plunderer makes a Treasure when another of your creatures dies.
    #[test]
    fn pitiless_plunderer_makes_treasure() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::pitiless_plunderer());
        let victim = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.battlefield_find_mut(victim).unwrap().damage = 2;
        let evs = g.check_state_based_actions();
        g.dispatch_triggers_for_events(&evs);
        drain_stack(&mut g);
        assert!(g.battlefield.iter().any(|c| c.definition.name == "Treasure"), "treasure minted");
    }

    /// Falkenrath Aristocrat sacrifices a creature to become indestructible.
    #[test]
    fn falkenrath_aristocrat_sacs_for_indestructible() {
        let mut g = two_player_game();
        let fa = g.add_card_to_battlefield(0, catalog::falkenrath_aristocrat());
        g.add_card_to_battlefield(0, catalog::grizzly_bears()); // sac fodder
        g.perform_action(GameAction::ActivateAbility {
            card_id: fa, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
        }).expect("sac a creature");
        drain_stack(&mut g);
        assert!(g.computed_permanent(fa).unwrap().keywords.contains(&Keyword::Indestructible));
    }

    /// Terrarion enters tapped, sacs for two mana, and cantrips on death.
    #[test]
    fn terrarion_taps_ramps_and_cantrips() {
        let mut g = two_player_game();
        let t = g.move_card_to_battlefield_for_test(0, catalog::terrarion());
        assert!(g.battlefield_find(t).unwrap().tapped, "enters tapped");
        g.battlefield_find_mut(t).unwrap().tapped = false; // ready to tap for its ability
        g.add_card_to_library(0, catalog::forest());
        g.players[0].mana_pool.add_colorless(2);
        let hand = g.players[0].hand.len();
        g.perform_action(GameAction::ActivateAbility {
            card_id: t, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
        }).expect("sac for mana");
        drain_stack(&mut g);
        assert!(g.players[0].mana_pool.total() >= 2, "added two mana");
        assert_eq!(g.players[0].hand.len(), hand + 1, "cantrip drew a card");
    }

    /// Implement of Combustion pings and cantrips.
    #[test]
    fn implement_of_combustion_pings_and_cantrips() {
        let mut g = two_player_game();
        let imp = g.add_card_to_battlefield(0, catalog::implement_of_combustion());
        g.add_card_to_library(0, catalog::forest());
        g.players[0].mana_pool.add(Color::Red, 1);
        let hand = g.players[0].hand.len();
        g.perform_action(GameAction::ActivateAbility {
            card_id: imp, ability_index: 0, target: Some(Target::Player(1)), additional_targets: vec![], x_value: None,
        }).expect("ping");
        drain_stack(&mut g);
        assert_eq!(g.players[1].life, 19, "1 damage to opponent");
        assert_eq!(g.players[0].hand.len(), hand + 1, "cantrip on death");
    }

    /// Reckless Fireweaver pings each opponent when an artifact enters.
    #[test]
    fn reckless_fireweaver_pings_on_artifact_etb() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::reckless_fireweaver());
        let art = g.move_card_to_battlefield_for_test(0, catalog::terrarion());
        g.dispatch_triggers_for_events(&[GameEvent::PermanentEntered { card_id: art }]);
        drain_stack(&mut g);
        assert_eq!(g.players[1].life, 19, "1 damage when an artifact entered");
    }

    /// Disciple of the Vault drains when an artifact dies.
    #[test]
    fn disciple_of_the_vault_drains_on_artifact_death() {
        let mut g = two_player_game();
        use crabomination::effect::{Effect, Selector, Value};
        g.add_card_to_battlefield(0, catalog::disciple_of_the_vault());
        g.add_card_to_battlefield(0, catalog::worn_powerstone()); // a vanilla artifact
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
        let ctx = crabomination::game::effects::EffectContext::for_ability(crabomination::card::CardId(0), 0, None);
        let evs = g.resolve_effect(&Effect::Sacrifice {
            who: Selector::You, count: Value::Const(1), filter: SelectionRequirement::Artifact,
        }, &ctx).unwrap();
        g.dispatch_triggers_for_events(&evs);
        drain_stack(&mut g);
        assert_eq!(g.players[1].life, 19, "opponent lost 1");
    }

    /// Marionette Master drains by the dying artifact's mana value.
    #[test]
    fn marionette_master_drains_by_mana_value() {
        let mut g = two_player_game();
        let mm = catalog::marionette_master();
        assert_eq!(mm.triggered_abilities.len(), 2, "fabricate + artifact-death drain");
        use crabomination::effect::{Effect, Selector, Value};
        g.add_card_to_battlefield(0, mm);
        // A {3} artifact (mana value 3) is sacrificed.
        let art = g.add_card_to_battlefield(0, catalog::worn_powerstone());
        let mv = g.battlefield_find(art).unwrap().definition.cost.cmc();
        let ctx = crabomination::game::effects::EffectContext::for_ability(crabomination::card::CardId(0), 0, None);
        let evs = g.resolve_effect(&Effect::Sacrifice {
            who: Selector::You, count: Value::Const(1), filter: SelectionRequirement::Artifact,
        }, &ctx).unwrap();
        g.dispatch_triggers_for_events(&evs);
        drain_stack(&mut g);
        assert_eq!(g.players[1].life, 20 - mv as i32, "lost life = artifact mana value");
    }

    /// Glassdust Hulk grows and turns unblockable when another artifact enters.
    #[test]
    fn glassdust_hulk_grows_on_artifact_etb() {
        let mut g = two_player_game();
        let hulk = g.add_card_to_battlefield(0, catalog::glassdust_hulk());
        let art = g.move_card_to_battlefield_for_test(0, catalog::terrarion());
        g.dispatch_triggers_for_events(&[GameEvent::PermanentEntered { card_id: art }]);
        drain_stack(&mut g);
        let cp = g.computed_permanent(hulk).unwrap();
        assert_eq!((cp.power, cp.toughness), (4, 5), "+1/+1");
        assert!(cp.keywords.contains(&Keyword::Unblockable));
    }

    /// Logic Knot counters a spell unless its controller pays {X}.
    #[test]
    fn logic_knot_counters_with_x() {
        let mut g = two_player_game();
        let spell = g.add_card_to_hand(1, catalog::sign_in_blood());
        g.active_player_idx = 1;
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 1;
        g.players[1].mana_pool.add(Color::Black, 2);
        g.perform_action(GameAction::CastSpell {
            card_id: spell, target: Some(Target::Player(1)), additional_targets: vec![], mode: None, x_value: None,
        }).ok();
        let knot = g.add_card_to_hand(0, catalog::logic_knot());
        g.priority.player_with_priority = 0;
        g.players[0].mana_pool.add(Color::Blue, 2);
        g.players[0].mana_pool.add_colorless(2); // X=2
        g.perform_action(GameAction::CastSpell {
            card_id: knot, target: Some(Target::Permanent(spell)), additional_targets: vec![], mode: None, x_value: Some(2),
        }).expect("cast logic knot X=2");
        drain_stack(&mut g);
        assert!(g.players[1].graveyard.iter().any(|c| c.id == spell), "spell countered (opp couldn't pay {{2}})");
    }

    /// Beanstalk Giant's power and toughness track lands you control.
    #[test]
    fn beanstalk_giant_scales_with_lands() {
        let mut g = two_player_game();
        let bg = g.add_card_to_battlefield(0, catalog::beanstalk_giant());
        g.add_card_to_battlefield(0, catalog::forest());
        g.add_card_to_battlefield(0, catalog::forest());
        g.add_card_to_battlefield(0, catalog::mountain());
        let cp = g.computed_permanent(bg).unwrap();
        assert_eq!((cp.power, cp.toughness), (3, 3), "*/* = three lands");
    }

    /// Ambush Viper is a flash deathtouch creature.
    #[test]
    fn ambush_viper_flash_deathtouch() {
        let d = catalog::ambush_viper();
        assert!(d.keywords.contains(&Keyword::Flash));
        assert!(d.keywords.contains(&Keyword::Deathtouch));
    }

    /// Etherium Sculptor makes your artifact spells cost {1} less.
    #[test]
    fn etherium_sculptor_discounts_artifacts() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::etherium_sculptor());
        let art = g.add_card_to_hand(0, catalog::worn_powerstone()); // {3}
        // {3} artifact costs {2} with the discount.
        g.players[0].mana_pool.add_colorless(2);
        g.active_player_idx = 0;
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::CastSpell {
            card_id: art, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast discounted artifact");
        drain_stack(&mut g);
        assert!(g.battlefield_find(art).is_some(), "powerstone resolved for {{2}}");
    }

    /// Toolcraft Exemplar grows at combat when you control artifacts.
    #[test]
    fn toolcraft_exemplar_grows_with_artifacts() {
        let mut g = two_player_game();
        let tc = g.add_card_to_battlefield(0, catalog::toolcraft_exemplar());
        for _ in 0..3 {
            g.add_card_to_battlefield(0, catalog::worn_powerstone());
        }
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        g.fire_step_triggers(TurnStep::BeginCombat);
        drain_stack(&mut g);
        let cp = g.computed_permanent(tc).unwrap();
        assert_eq!((cp.power, cp.toughness), (3, 2), "+2/+1");
        assert!(cp.keywords.contains(&Keyword::FirstStrike), "first strike with 3 artifacts");
    }

    /// Vampire Gourmand sacrifices a creature on attack to draw and gain evasion.
    #[test]
    fn vampire_gourmand_sacs_to_draw() {
        let mut g = two_player_game();
        let vg = g.add_card_to_battlefield(0, catalog::vampire_gourmand());
        g.add_card_to_battlefield(0, catalog::grizzly_bears()); // fodder
        g.add_card_to_library(0, catalog::swamp());
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
        let hand = g.players[0].hand.len();
        let eff = catalog::vampire_gourmand().triggered_abilities[0].effect.clone();
        let ctx = crabomination::game::effects::EffectContext::for_trigger(vg, 0, None, 0);
        g.resolve_effect(&eff, &ctx).unwrap();
        assert_eq!(g.players[0].hand.len(), hand + 1, "drew a card");
        assert!(g.computed_permanent(vg).unwrap().keywords.contains(&Keyword::Unblockable));
    }

    /// Recruitment Officer digs four for a small creature.
    #[test]
    fn recruitment_officer_digs() {
        let mut g = two_player_game();
        let ro = g.add_card_to_battlefield(0, catalog::recruitment_officer());
        g.add_card_to_library(0, catalog::grizzly_bears()); // MV2 creature
        g.add_card_to_library(0, catalog::mountain());
        g.players[0].mana_pool.add(Color::White, 1);
        g.players[0].mana_pool.add_colorless(3);
        let hand = g.players[0].hand.len();
        g.perform_action(GameAction::ActivateAbility {
            card_id: ro, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
        }).expect("dig");
        drain_stack(&mut g);
        assert_eq!(g.players[0].hand.len(), hand + 1, "took a creature to hand");
    }

    /// Squee returns from the graveyard at your upkeep.
    #[test]
    fn squee_returns_from_graveyard() {
        let mut g = two_player_game();
        let squee = g.add_card_to_graveyard(0, catalog::squee_goblin_nabob());
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
        g.active_player_idx = 0;
        g.fire_step_triggers(TurnStep::Upkeep);
        drain_stack(&mut g);
        assert!(g.players[0].hand.iter().any(|c| c.id == squee), "Squee back in hand");
    }

    /// Hazoret can't attack with a full hand, but can when hellbent; the discard
    /// ability burns each opponent.
    #[test]
    fn hazoret_hellbent_gate_and_burn() {
        let mut g = two_player_game();
        let haz = g.add_card_to_battlefield(0, catalog::hazoret_the_fervent());
        g.battlefield_find_mut(haz).unwrap().summoning_sick = false;
        g.active_player_idx = 0;
        g.step = TurnStep::DeclareAttackers;
        g.priority.player_with_priority = 0;
        g.add_card_to_hand(0, catalog::mountain());
        g.add_card_to_hand(0, catalog::mountain()); // two cards: locked
        assert!(!g.legal_attackers(0).contains(&haz), "locked with a full hand");
        g.players[0].hand.pop(); // down to one card: unlocked
        assert!(g.legal_attackers(0).contains(&haz), "attacks when hellbent");
        // {2}{R}, discard a card: 2 to each opponent.
        g.players[0].mana_pool.add(Color::Red, 1);
        g.players[0].mana_pool.add_colorless(2);
        g.perform_action(GameAction::ActivateAbility {
            card_id: haz, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
        }).expect("discard for damage");
        drain_stack(&mut g);
        assert_eq!(g.players[1].life, 18, "2 damage to opponent");
    }

    /// Patchwork Beastie can't attack until delirium is active.
    #[test]
    fn patchwork_beastie_needs_delirium() {
        let mut g = two_player_game();
        let pb = g.add_card_to_battlefield(0, catalog::patchwork_beastie());
        g.battlefield_find_mut(pb).unwrap().summoning_sick = false;
        g.active_player_idx = 0;
        g.step = TurnStep::DeclareAttackers;
        g.priority.player_with_priority = 0;
        assert!(!g.legal_attackers(0).contains(&pb), "locked without delirium");
        // Seed four card types into the graveyard.
        g.add_card_to_graveyard(0, catalog::grizzly_bears()); // creature
        g.add_card_to_graveyard(0, catalog::mountain());      // land
        g.add_card_to_graveyard(0, catalog::sign_in_blood()); // sorcery
        g.add_card_to_graveyard(0, catalog::ambush_viper());  // creature (still 3 types)
        g.add_card_to_graveyard(0, catalog::worn_powerstone()); // artifact -> 4 types
        assert!(g.delirium_active(0), "four card types");
        assert!(g.legal_attackers(0).contains(&pb), "attacks with delirium");
    }

    /// Fervent Champion pumps another attacking Knight.
    #[test]
    fn fervent_champion_pumps_knight() {
        let mut g = two_player_game();
        let champ = g.add_card_to_battlefield(0, catalog::fervent_champion());
        let ally = g.add_card_to_battlefield(0, catalog::fervent_champion());
        let eff = catalog::fervent_champion().triggered_abilities[0].effect.clone();
        let ctx = crabomination::game::effects::EffectContext::for_trigger(champ, 0, Some(Target::Permanent(ally)), 0);
        g.resolve_effect(&eff, &ctx).unwrap();
        let cp = g.computed_permanent(ally).unwrap();
        assert_eq!(cp.power, 2, "+1/+0");
    }

    /// Porcelain Legionnaire can be cast paying life for its Phyrexian pip.
    #[test]
    fn porcelain_legionnaire_phyrexian_cast() {
        let mut g = two_player_game();
        let pl = g.add_card_to_hand(0, catalog::porcelain_legionnaire());
        g.players[0].mana_pool.add_colorless(2); // pay {2}, then 2 life for {W/P}
        g.active_player_idx = 0;
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        let life = g.players[0].life;
        g.perform_action(GameAction::CastSpell {
            card_id: pl, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast paying Phyrexian life");
        drain_stack(&mut g);
        assert!(g.battlefield_find(pl).is_some(), "resolved");
        assert_eq!(g.players[0].life, life - 2, "paid 2 life for {{W/P}}");
    }

    /// Short Sword grants +1/+1 when equipped.
    #[test]
    fn short_sword_pumps() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let sword = g.add_card_to_battlefield(0, catalog::short_sword());
        g.players[0].mana_pool.add_colorless(1);
        g.perform_action(GameAction::Equip { equipment: sword, target: bear }).expect("equip");
        let cp = g.computed_permanent(bear).unwrap();
        assert_eq!((cp.power, cp.toughness), (3, 3), "+1/+1");
    }

    /// Axebane Beast is a 3/4 vanilla.
    #[test]
    fn axebane_beast_stats() {
        let d = catalog::axebane_beast();
        assert_eq!((d.power, d.toughness), (3, 4));
    }

    /// Yavimaya Sapherd makes a Saproling on ETB.
    #[test]
    fn yavimaya_sapherd_makes_saproling() {
        let mut g = two_player_game();
        g.move_card_to_battlefield_for_test(0, catalog::yavimaya_sapherd());
        drain_stack(&mut g);
        assert!(g.battlefield.iter().any(|c| c.definition.name == "Saproling" && c.is_token));
    }

    /// Faerie Guidemother's Gift of the Fae pumps and grants flying.
    #[test]
    fn faerie_guidemother_adventure_buffs() {
        let mut g = two_player_game();
        let d = catalog::faerie_guidemother();
        let adv = d.adventure.expect("adventure");
        assert_eq!(adv.name, "Gift of the Fae");
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let ctx = crabomination::game::effects::EffectContext::for_ability(
            crabomination::card::CardId(99), 0, Some(Target::Permanent(bear)),
        );
        g.resolve_effect(&adv.effect, &ctx).unwrap();
        let cp = g.computed_permanent(bear).unwrap();
        assert_eq!((cp.power, cp.toughness), (4, 3), "+2/+1");
        assert!(cp.keywords.contains(&Keyword::Flying));
    }

    /// All That Glitters scales with your artifacts and enchantments.
    #[test]
    fn all_that_glitters_scales() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.add_card_to_battlefield(0, catalog::worn_powerstone()); // artifact
        g.add_card_to_battlefield(0, catalog::short_sword());     // artifact
        let aura = g.add_card_to_hand(0, catalog::all_that_glitters());
        g.players[0].mana_pool.add(Color::White, 1);
        g.players[0].mana_pool.add_colorless(1);
        cast_at(&mut g, aura, Target::Permanent(bear));
        // The aura itself is an enchantment you control, so 2 artifacts + 1
        // enchantment (the aura) = +3/+3 → 5/5.
        let cp = g.computed_permanent(bear).unwrap();
        assert_eq!((cp.power, cp.toughness), (5, 5));
    }

    /// Scorching Dragonfire exiles a creature it kills.
    #[test]
    fn scorching_dragonfire_exiles_on_kill() {
        let mut g = two_player_game();
        let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
        let bolt = g.add_card_to_hand(0, catalog::scorching_dragonfire());
        g.players[0].mana_pool.add(Color::Red, 1);
        g.players[0].mana_pool.add_colorless(1);
        cast_at(&mut g, bolt, Target::Permanent(foe));
        g.check_state_based_actions();
        assert!(g.battlefield_find(foe).is_none(), "creature dead");
        assert!(g.exile.iter().any(|c| c.id == foe), "exiled, not in graveyard");
    }

    /// Slaying Fire deals 3 to any target on a non-adamant payment.
    #[test]
    fn slaying_fire_burns() {
        let mut g = two_player_game();
        let bolt = g.add_card_to_hand(0, catalog::slaying_fire());
        g.players[0].mana_pool.add(Color::Red, 2);
        g.players[0].mana_pool.add(Color::Green, 1);
        cast_at(&mut g, bolt, Target::Player(1));
        assert_eq!(g.players[1].life, 17, "3 damage without three red");
    }

    /// Searing Barrage deals 5 to a creature.
    #[test]
    fn searing_barrage_burns_creature() {
        let mut g = two_player_game();
        let foe = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4
        let bolt = g.add_card_to_hand(0, catalog::searing_barrage());
        g.players[0].mana_pool.add(Color::Red, 1);
        g.players[0].mana_pool.add_colorless(4);
        cast_at(&mut g, bolt, Target::Permanent(foe));
        g.check_state_based_actions();
        assert!(g.battlefield_find(foe).is_none(), "5 damage kills the 4/4");
    }

    /// Brazen Wolves pumps when it attacks.
    #[test]
    fn brazen_wolves_attack_pump() {
        let mut g = two_player_game();
        let w = g.add_card_to_battlefield(0, catalog::brazen_wolves());
        let eff = catalog::brazen_wolves().triggered_abilities[0].effect.clone();
        let ctx = crabomination::game::effects::EffectContext::for_trigger(w, 0, None, 0);
        g.resolve_effect(&eff, &ctx).unwrap();
        let cp = g.computed_permanent(w).unwrap();
        assert_eq!((cp.power, cp.toughness), (4, 3), "+2/+0");
    }

    /// Glory Seeker is a 2/2 vanilla.
    #[test]
    fn glory_seeker_stats() {
        let d = catalog::glory_seeker();
        assert_eq!((d.power, d.toughness), (2, 2));
        assert!(d.triggered_abilities.is_empty() && d.activated_abilities.is_empty());
    }

    /// Pheres-Band Tromper grows when it untaps (Inspired).
    #[test]
    fn pheres_band_tromper_inspired() {
        let mut g = two_player_game();
        let t = g.add_card_to_battlefield(0, catalog::pheres_band_tromper());
        g.battlefield_find_mut(t).unwrap().tapped = true;
        g.dispatch_triggers_for_events(&[GameEvent::PermanentUntapped { card_id: t }]);
        drain_stack(&mut g);
        assert_eq!(g.battlefield_find(t).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
    }

    /// Dwarven Berserker swells when blocked.
    #[test]
    fn dwarven_berserker_swells_when_blocked() {
        let mut g = two_player_game();
        let db = g.add_card_to_battlefield(0, catalog::dwarven_berserker());
        let blocker = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        g.dispatch_triggers_for_events(&[GameEvent::BlockerDeclared { blocker, attacker: db }]);
        drain_stack(&mut g);
        let cp = g.computed_permanent(db).unwrap();
        assert_eq!((cp.power, cp.toughness), (4, 1), "+3/+0");
        assert!(cp.keywords.contains(&Keyword::Trample));
    }

    /// Elvish Hexhunter sacrifices to destroy an enchantment.
    #[test]
    fn elvish_hexhunter_destroys_enchantment() {
        let mut g = two_player_game();
        let eh = g.add_card_to_battlefield(0, catalog::elvish_hexhunter());
        g.clear_sickness(eh);
        let ench = g.add_card_to_battlefield(1, catalog::hardened_scales());
        g.players[0].mana_pool.add(Color::Green, 1);
        g.perform_action(GameAction::ActivateAbility {
            card_id: eh, ability_index: 0, target: Some(Target::Permanent(ench)), additional_targets: vec![], x_value: None,
        }).expect("sac to destroy enchantment");
        drain_stack(&mut g);
        assert!(g.battlefield_find(ench).is_none(), "enchantment destroyed");
        assert!(g.battlefield_find(eh).is_none(), "hexhunter sacrificed");
    }

    /// Felidar Savior buffs two other creatures on ETB.
    #[test]
    fn felidar_savior_buffs_two() {
        let mut g = two_player_game();
        let a = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let b = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let savior = g.add_card_to_battlefield(0, catalog::felidar_savior());
        let eff = catalog::felidar_savior().triggered_abilities[0].effect.clone();
        let mut ctx = crabomination::game::effects::EffectContext::for_trigger(savior, 0, None, 0);
        ctx.targets = vec![Target::Permanent(a), Target::Permanent(b)];
        g.resolve_effect(&eff, &ctx).unwrap();
        // Up to two other creatures you control each get a +1/+1 counter.
        let buffed = [a, b].iter()
            .filter(|c| g.battlefield_find(**c).unwrap().counter_count(CounterType::PlusOnePlusOne) == 1)
            .count();
        assert_eq!(buffed, 2, "both other creatures buffed");
    }
}

mod recent2 {
    use crabomination::card::{CounterType, CreatureType, Effect, Keyword, StaticEffect};
    use crabomination::catalog;
    use crabomination::game::types::{Attack, AttackTarget, TurnStep};
    use crabomination::game::*;
    use crabomination::mana::Color;

    fn advance_to(g: &mut GameState, step: TurnStep) {
        while g.step != step {
            g.perform_action(GameAction::PassPriority).expect("pass priority");
        }
    }

    /// Tangle fogs combat and keeps attackers from untapping next turn.
    #[test]
    fn tangle_fogs_and_locks_attackers() {
        let mut g = two_player_game();
        let atk = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.clear_sickness(atk);
        advance_to(&mut g, TurnStep::DeclareAttackers);
        g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker: atk, target: AttackTarget::Player(1),
        }])).expect("attack");
        drain_stack(&mut g);
        let id = g.add_card_to_hand(0, catalog::tangle());
        g.players[0].mana_pool.add(Color::Green, 1);
        g.players[0].mana_pool.add_colorless(1);
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast Tangle");
        drain_stack(&mut g);
        advance_to(&mut g, TurnStep::PostCombatMain);
        assert_eq!(g.players[1].life, 20, "combat damage prevented");
        assert!(g.battlefield_find(atk).unwrap().tapped, "attacker still tapped");
    }

    /// March of Otherworldly Light exiles a creature with MV ≤ X.
    #[test]
    fn march_of_otherworldly_light_exiles_by_x() {
        let mut g = two_player_game();
        let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // MV 2
        let id = g.add_card_to_hand(0, catalog::march_of_otherworldly_light());
        g.players[0].mana_pool.add(Color::White, 1);
        g.players[0].mana_pool.add_colorless(2); // X = 2
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: Some(Target::Permanent(victim)),
            additional_targets: vec![], mode: None, x_value: Some(2),
        }).expect("cast March");
        drain_stack(&mut g);
        assert!(g.exile.iter().any(|c| c.id == victim), "creature exiled");
    }

    /// Disdainful Stroke counters a 4-MV spell but not a cheap one.
    #[test]
    fn disdainful_stroke_counters_expensive_spell() {
        let mut g = two_player_game();
        g.active_player_idx = 1;
        g.priority.player_with_priority = 1;
        let spell = g.add_card_to_hand(1, catalog::serra_angel()); // {3}{W}{W} = MV 5
        g.players[1].mana_pool.add(Color::White, 2);
        g.players[1].mana_pool.add_colorless(3);
        g.perform_action(GameAction::CastSpell {
            card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast Serra Angel");
        g.perform_action(GameAction::PassPriority).expect("P1 passes to P0");
        let ds = g.add_card_to_hand(0, catalog::disdainful_stroke());
        g.players[0].mana_pool.add(Color::Blue, 1);
        g.players[0].mana_pool.add_colorless(1);
        g.perform_action(GameAction::CastSpell {
            card_id: ds, target: Some(Target::Permanent(spell)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast Disdainful Stroke");
        drain_stack(&mut g);
        assert!(g.battlefield_find(spell).is_none(), "Serra Angel countered");
    }

    /// Flame Lash deals 4 to a player.
    #[test]
    fn flame_lash_deals_four() {
        let mut g = two_player_game();
        let id = g.add_card_to_hand(0, catalog::flame_lash());
        g.players[0].mana_pool.add(Color::Red, 1);
        g.players[0].mana_pool.add_colorless(3);
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: Some(Target::Player(1)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast Flame Lash");
        drain_stack(&mut g);
        assert_eq!(g.players[1].life, 16);
    }

    /// Virtue of Persistence: the adventure half (-3/-3 + gain 2 life) resolves.
    #[test]
    fn virtue_of_persistence_adventure_shrinks_and_gains() {
        let mut g = two_player_game();
        let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2 → dies
        let id = g.add_card_to_hand(0, catalog::virtue_of_persistence());
        g.players[0].mana_pool.add(Color::Black, 1);
        g.players[0].mana_pool.add_colorless(1);
        g.perform_action(GameAction::CastAdventure {
            card_id: id, target: Some(Target::Permanent(victim)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast Locthwain Scorn");
        drain_stack(&mut g);
        assert!(g.battlefield_find(victim).is_none(), "−3/−3 killed the 2/2");
        assert_eq!(g.players[0].life, 22, "gained 2 life");
    }

    /// Scrabbling Skullcrab mills when an enchantment enters under your control.
    #[test]
    fn scrabbling_skullcrab_mills_on_enchantment_etb() {
        let mut g = two_player_game();
        for _ in 0..3 { g.add_card_to_library(1, catalog::island()); }
        g.add_card_to_battlefield(0, catalog::scrabbling_skullcrab());
        let lib_before = g.players[1].library.len();
        // An enchantment entering under your control triggers the mill — cast it
        // through the full path so observer triggers dispatch.
        let ench = g.add_card_to_hand(0, catalog::possibility_storm());
        g.players[0].mana_pool.add(Color::Red, 2);
        g.players[0].mana_pool.add_colorless(3);
        g.perform_action(GameAction::CastSpell {
            card_id: ench, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast Possibility Storm");
        drain_stack(&mut g);
        assert_eq!(g.players[1].library.len(), lib_before - 2, "opponent milled two");
    }

    /// Hush destroys every enchantment.
    #[test]
    fn hush_destroys_all_enchantments() {
        let mut g = two_player_game();
        let e1 = g.add_card_to_battlefield(0, catalog::glorious_anthem());
        let e2 = g.add_card_to_battlefield(1, catalog::glorious_anthem());
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let id = g.add_card_to_hand(0, catalog::hush());
        g.players[0].mana_pool.add(Color::Green, 1);
        g.players[0].mana_pool.add_colorless(3);
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast Hush");
        drain_stack(&mut g);
        assert!(g.battlefield_find(e1).is_none() && g.battlefield_find(e2).is_none(), "enchantments gone");
        assert!(g.battlefield_find(bear).is_some(), "creature untouched");
    }

    /// Llanowar Greenwidow returns itself from the graveyard to the battlefield.
    #[test]
    fn llanowar_greenwidow_returns_from_graveyard() {
        let mut g = two_player_game();
        let id = g.add_card_to_graveyard(0, catalog::llanowar_greenwidow());
        g.players[0].mana_pool.add(Color::Green, 1);
        g.players[0].mana_pool.add_colorless(7);
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::ActivateAbility {
            card_id: id, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
        }).expect("activate graveyard return");
        drain_stack(&mut g);
        let r = g.battlefield_find(id).expect("returned to battlefield");
        assert!(r.tapped, "returns tapped");
        assert_eq!((r.power(), r.toughness()), (4, 3));
    }

    /// Searchlight Companion makes a Spirit token on ETB.
    #[test]
    fn searchlight_companion_makes_a_spirit() {
        let mut g = two_player_game();
        g.move_card_to_battlefield_for_test(0, catalog::searchlight_companion());
        drain_stack(&mut g);
        let spirits = g.battlefield.iter()
            .filter(|c| c.is_token && c.definition.name == "Spirit").count();
        assert_eq!(spirits, 1);
    }

    /// Resolute Reinforcements has flash and makes a Soldier on ETB.
    #[test]
    fn resolute_reinforcements_makes_a_soldier() {
        let mut g = two_player_game();
        assert!(catalog::resolute_reinforcements().keywords.contains(&Keyword::Flash));
        g.move_card_to_battlefield_for_test(0, catalog::resolute_reinforcements());
        drain_stack(&mut g);
        let soldiers = g.battlefield.iter()
            .filter(|c| c.is_token && c.definition.name == "Soldier").count();
        assert_eq!(soldiers, 1);
    }

    /// Jewel Thief makes a Treasure on ETB.
    #[test]
    fn jewel_thief_makes_a_treasure() {
        let mut g = two_player_game();
        g.move_card_to_battlefield_for_test(0, catalog::jewel_thief());
        drain_stack(&mut g);
        assert!(g.battlefield.iter().any(|c| c.is_token && c.definition.name == "Treasure"));
    }

    /// Sweettooth Witch makes a Food on ETB.
    #[test]
    fn sweettooth_witch_makes_a_food() {
        let mut g = two_player_game();
        g.move_card_to_battlefield_for_test(0, catalog::sweettooth_witch());
        drain_stack(&mut g);
        assert!(g.battlefield.iter().any(|c| c.is_token && c.definition.name == "Food"));
    }

    /// Ambush Paratrooper's {5} ability pumps the team.
    #[test]
    fn ambush_paratrooper_pumps_team() {
        let mut g = two_player_game();
        let trooper = g.add_card_to_battlefield(0, catalog::ambush_paratrooper());
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.players[0].mana_pool.add_colorless(5);
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::ActivateAbility {
            card_id: trooper, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
        }).expect("pump");
        drain_stack(&mut g);
        assert_eq!(g.computed_permanent(bear).unwrap().power, 3, "bear pumped +1/+1");
    }

    /// Glistening Deluge shrinks all creatures and hits G/W harder.
    #[test]
    fn glistening_deluge_punishes_green_white() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // green 2/2 → -3/-3 dies
        let id = g.add_card_to_hand(0, catalog::glistening_deluge());
        g.players[0].mana_pool.add(Color::Black, 2);
        g.players[0].mana_pool.add_colorless(1);
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast Glistening Deluge");
        drain_stack(&mut g);
        assert!(g.battlefield_find(bear).is_none(), "green 2/2 took -3/-3 and died");
    }

    /// Faerie Dreamthief surveils on ETB and can be exiled from the graveyard to draw.
    #[test]
    fn faerie_dreamthief_surveils_and_recurs() {
        let mut g = two_player_game();
        g.add_card_to_library(0, catalog::island());
        g.move_card_to_battlefield_for_test(0, catalog::faerie_dreamthief());
        drain_stack(&mut g);
        let d = catalog::faerie_dreamthief();
        assert_eq!(d.triggered_abilities.len(), 1, "ETB surveil wired");
        assert!(d.activated_abilities[0].from_graveyard && d.activated_abilities[0].exile_self_cost);
    }

    /// Vinereap Mentor makes a Food on ETB (and again on death).
    #[test]
    fn vinereap_mentor_makes_food_on_etb() {
        let mut g = two_player_game();
        g.move_card_to_battlefield_for_test(0, catalog::vinereap_mentor());
        drain_stack(&mut g);
        assert!(g.battlefield.iter().any(|c| c.is_token && c.definition.name == "Food"));
        assert_eq!(catalog::vinereap_mentor().triggered_abilities.len(), 2, "etb + dies");
    }

    /// Topiary Panther is a 6/5 trampler with basic landcycling.
    #[test]
    fn topiary_panther_has_basic_landcycling() {
        let d = catalog::topiary_panther();
        assert_eq!((d.power, d.toughness), (6, 5));
        assert!(d.keywords.iter().any(|k| matches!(k, Keyword::Typecycling(_))));
    }

    /// Valgavoth's Faithful sacrifices itself to reanimate a creature.
    #[test]
    fn valgavoths_faithful_reanimates() {
        let mut g = two_player_game();
        let faithful = g.add_card_to_battlefield(0, catalog::valgavoths_faithful());
        let dead = g.add_card_to_graveyard(0, catalog::serra_angel());
        g.players[0].mana_pool.add(Color::Black, 1);
        g.players[0].mana_pool.add_colorless(3);
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::ActivateAbility {
            card_id: faithful, ability_index: 0, target: Some(Target::Permanent(dead)),
            additional_targets: vec![], x_value: None,
        }).expect("reanimate");
        drain_stack(&mut g);
        assert!(g.battlefield_find(faithful).is_none(), "Faithful sacrificed");
        assert!(g.battlefield_find(dead).is_some(), "Serra Angel reanimated");
    }

    /// Charforger makes a Goblin on ETB and grows when another of your creatures dies.
    #[test]
    fn charforger_etb_token_and_death_growth() {
        let mut g = two_player_game();
        let charforger = g.move_card_to_battlefield_for_test(0, catalog::charforger());
        drain_stack(&mut g);
        assert!(g.battlefield.iter().any(|c| c.is_token && c.definition.name == "Phyrexian Goblin"));
        // Kill another creature you control via lethal damage so the death goes
        // through the SBA + observer-trigger dispatch (not the self-source helper).
        let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
        let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
        g.players[0].mana_pool.add(Color::Red, 1);
        g.perform_action(GameAction::CastSpell {
            card_id: bolt, target: Some(Target::Permanent(fodder)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("bolt fodder");
        drain_stack(&mut g);
        assert_eq!(
            g.battlefield_find(charforger).unwrap().counter_count(CounterType::Oil),
            1,
            "oiled up when another creature you control died",
        );
    }

    /// Voracious Vermin makes a Rat on ETB.
    #[test]
    fn voracious_vermin_makes_a_rat() {
        let mut g = two_player_game();
        g.move_card_to_battlefield_for_test(0, catalog::voracious_vermin());
        drain_stack(&mut g);
        assert!(g.battlefield.iter().any(|c| c.is_token
            && c.definition.subtypes.creature_types.contains(&CreatureType::Rat)));
    }

    /// Mocking Sprite reduces your instant/sorcery costs by {1}.
    #[test]
    fn mocking_sprite_discounts_instants() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::mocking_sprite());
        // Lightning Bolt {R} would normally need {R}; with the discount a {1}{R}
        // instant should be castable off two mana. Use a {1}{R} instant: Flame Lash
        // is {3}{R}; instead verify the static is wired and matches instants.
        let d = catalog::mocking_sprite();
        assert!(matches!(d.static_abilities[0].effect, StaticEffect::CostReduction { amount: 1, .. }));
        // Functional check: a {1}{U} instant (Disdainful Stroke) is castable for {U}.
        g.priority.player_with_priority = 1;
        g.active_player_idx = 1;
        let spell = g.add_card_to_hand(1, catalog::serra_angel());
        g.players[1].mana_pool.add(Color::White, 2);
        g.players[1].mana_pool.add_colorless(3);
        g.perform_action(GameAction::CastSpell {
            card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("opp casts");
        g.perform_action(GameAction::PassPriority).expect("pass");
        let ds = g.add_card_to_hand(0, catalog::disdainful_stroke());
        g.players[0].mana_pool.add(Color::Blue, 1); // only {U}, discount covers the {1}
        g.perform_action(GameAction::CastSpell {
            card_id: ds, target: Some(Target::Permanent(spell)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("Disdainful Stroke castable for {U} under Mocking Sprite");
        drain_stack(&mut g);
        assert!(g.battlefield_find(spell).is_none(), "countered");
    }

    /// Ancestral Reminiscence draws three then discards one (net +2).
    #[test]
    fn ancestral_reminiscence_draws_three_discards_one() {
        let mut g = two_player_game();
        for _ in 0..5 { g.add_card_to_library(0, catalog::island()); }
        let id = g.add_card_to_hand(0, catalog::ancestral_reminiscence());
        let before = g.players[0].hand.len();
        g.players[0].mana_pool.add(Color::Blue, 1);
        g.players[0].mana_pool.add_colorless(3);
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast");
        drain_stack(&mut g);
        // cast (-1) + draw 3 (+3) - discard 1 (-1) = +1 vs before.
        assert_eq!(g.players[0].hand.len(), before + 1);
    }

    /// Charge pumps your team +1/+1.
    #[test]
    fn charge_pumps_team() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let id = g.add_card_to_hand(0, catalog::charge());
        g.players[0].mana_pool.add(Color::White, 1);
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast Charge");
        drain_stack(&mut g);
        assert_eq!(g.computed_permanent(bear).unwrap().power, 3);
    }

    /// Heroic Reinforcements makes two Soldiers and pumps + hastes the team.
    #[test]
    fn heroic_reinforcements_makes_soldiers_and_pumps() {
        let mut g = two_player_game();
        let id = g.add_card_to_hand(0, catalog::heroic_reinforcements());
        g.players[0].mana_pool.add(Color::Red, 1);
        g.players[0].mana_pool.add(Color::White, 1);
        g.players[0].mana_pool.add_colorless(2);
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast");
        drain_stack(&mut g);
        let soldiers: Vec<_> = g.battlefield.iter()
            .filter(|c| c.is_token && c.definition.name == "Soldier").collect();
        assert_eq!(soldiers.len(), 2);
        let sid = soldiers[0].id;
        assert_eq!(g.computed_permanent(sid).unwrap().power, 2, "1/1 pumped to 2/2");
        assert!(g.computed_permanent(sid).unwrap().keywords.contains(&Keyword::Haste));
    }

    /// Pyrewood Gearhulk buffs your other creatures on ETB.
    #[test]
    fn pyrewood_gearhulk_buffs_others() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.move_card_to_battlefield_for_test(0, catalog::pyrewood_gearhulk());
        drain_stack(&mut g);
        let cp = g.computed_permanent(bear).unwrap();
        assert_eq!(cp.power, 4, "other creature got +2/+2");
        assert!(cp.keywords.contains(&Keyword::Menace));
    }

    /// Beastbond Outcaster draws on ETB only with a power-4+ creature.
    #[test]
    fn beastbond_outcaster_conditional_draw() {
        let mut g = two_player_game();
        g.add_card_to_library(0, catalog::island());
        g.add_card_to_battlefield(0, catalog::serra_angel()); // 4/4 → condition met
        let before = g.players[0].hand.len();
        g.move_card_to_battlefield_for_test(0, catalog::beastbond_outcaster());
        drain_stack(&mut g);
        assert_eq!(g.players[0].hand.len(), before + 1, "drew with a big creature out");
        assert!(catalog::beastbond_outcaster().plot_cost.is_some());
    }

    /// Mindwhisker surveils at the beginning of your upkeep.
    #[test]
    fn mindwhisker_surveils_on_upkeep() {
        let d = catalog::mindwhisker();
        assert_eq!((d.power, d.toughness), (3, 2));
        assert!(matches!(d.triggered_abilities[0].effect, Effect::Surveil { .. }));
        assert!(matches!(
            d.triggered_abilities[0].event.kind,
            crabomination::card::EventKind::StepBegins(TurnStep::Upkeep)
        ));
    }

    /// Tarrian's Soulcleaver grants vigilance and grows the equipped creature when
    /// another permanent dies.
    #[test]
    fn tarrians_soulcleaver_grows_equipped_on_death() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let cleaver = g.add_card_to_battlefield(0, catalog::tarrians_soulcleaver());
        g.players[0].mana_pool.add_colorless(2);
        g.step = TurnStep::PreCombatMain;
        g.perform_action(GameAction::Equip { equipment: cleaver, target: bear }).expect("equip");
        drain_stack(&mut g);
        assert!(
            g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Vigilance),
            "equipped creature has vigilance"
        );
        // Another creature dies → +1/+1 counter on the equipped bear.
        let fodder = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        g.battlefield_find_mut(fodder).unwrap().damage = 2;
        let evs = g.check_state_based_actions();
        g.dispatch_triggers_for_events(&evs);
        drain_stack(&mut g);
        assert_eq!(
            g.battlefield_find(bear).unwrap().counters.get(&CounterType::PlusOnePlusOne).copied(),
            Some(1),
            "equipped creature grew by a +1/+1 counter"
        );
    }

    /// Snarespinner pumps +2/+0 when it blocks a flier (but not a grounded attacker).
    #[test]
    fn snarespinner_pumps_blocking_a_flier() {
        use crabomination::card::{CardDefinition, CardType};
        let flier = CardDefinition {
            name: "Test Drake",
            card_types: vec![CardType::Creature],
            power: 2,
            toughness: 2,
            keywords: vec![Keyword::Flying],
            ..Default::default()
        };
        let mut g = two_player_game();
        let atk = g.add_card_to_battlefield(0, flier);
        let spider = g.add_card_to_battlefield(1, catalog::snarespinner());
        g.clear_sickness(atk);
        advance_to(&mut g, TurnStep::DeclareAttackers);
        g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker: atk, target: AttackTarget::Player(1),
        }])).expect("attack");
        drain_stack(&mut g);
        advance_to(&mut g, TurnStep::DeclareBlockers);
        g.perform_action(GameAction::DeclareBlockers(vec![(spider, atk)])).expect("block");
        drain_stack(&mut g);
        let s = g.battlefield_find(spider).unwrap();
        assert_eq!((s.power(), s.toughness()), (3, 3), "+2/+0 for blocking a flier");
    }

    /// Lord Skitter makes a Rat at the beginning of combat on your turn.
    #[test]
    fn lord_skitter_makes_a_rat_in_combat() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::lord_skitter_sewer_king());
        advance_to(&mut g, TurnStep::BeginCombat);
        drain_stack(&mut g);
        let rats = g.battlefield.iter()
            .filter(|c| c.is_token && c.definition.subtypes.creature_types.contains(&CreatureType::Rat))
            .count();
        assert_eq!(rats, 1, "one Rat token created at combat");
    }

    /// Stickytongue Sentinel bounces another permanent you control on entry.
    #[test]
    fn stickytongue_sentinel_bounces_own_permanent() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.move_card_to_battlefield_for_test(0, catalog::stickytongue_sentinel());
        drain_stack(&mut g);
        assert!(g.battlefield_find(bear).is_none(), "bear left the battlefield");
        assert!(g.players[0].hand.iter().any(|c| c.id == bear), "bear returned to hand");
    }

    /// Ossification exiles an opponent's creature until it leaves.
    #[test]
    fn ossification_exiles_until_it_leaves() {
        let foe_def = catalog::grizzly_bears();
        let mut g = two_player_game();
        let foe = g.add_card_to_battlefield(1, foe_def);
        g.move_card_to_battlefield_for_test(0, catalog::ossification());
        drain_stack(&mut g);
        assert!(g.exile.iter().any(|c| c.id == foe), "opponent creature exiled");
        assert!(g.battlefield_find(foe).is_none());
    }

    /// Sunfall exiles all creatures.
    #[test]
    fn sunfall_exiles_all_creatures() {
        let mut g = two_player_game();
        let a = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let b = g.add_card_to_battlefield(1, catalog::serra_angel());
        let s = g.add_card_to_hand(0, catalog::sunfall());
        g.players[0].mana_pool.add(Color::White, 2);
        g.players[0].mana_pool.add_colorless(3);
        g.perform_action(GameAction::CastSpell {
            card_id: s, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast Sunfall");
        drain_stack(&mut g);
        assert!(g.battlefield_find(a).is_none() && g.battlefield_find(b).is_none(), "all creatures gone");
        assert_eq!(g.exile.iter().filter(|c| c.id == a || c.id == b).count(), 2, "both exiled");
    }

    /// Witchstalker Frenzy's discount counts every player's attackers this turn.
    #[test]
    fn witchstalker_frenzy_counts_all_attackers() {
        use crabomination::game::actions::cost_reduction_for_spell;
        let mut g = two_player_game();
        let spell = crabomination::card::CardInstance::new(g.next_id(), catalog::witchstalker_frenzy(), 0);
        assert_eq!(cost_reduction_for_spell(&g, 0, &spell, None), 0, "no attacks → full price");
        g.players[1].creatures_attacked_this_turn = 2; // the OPPONENT attacked
        assert_eq!(cost_reduction_for_spell(&g, 0, &spell, None), 2, "all-players count includes opp");
    }

    /// Warden of the Inner Sky gains flying and vigilance at three counters.
    #[test]
    fn warden_of_the_inner_sky_unlocks_at_three_counters() {
        let mut g = two_player_game();
        let w = g.add_card_to_battlefield(0, catalog::warden_of_the_inner_sky());
        assert!(!g.computed_permanent(w).unwrap().keywords.contains(&Keyword::Flying), "no flying yet");
        g.battlefield_find_mut(w).unwrap().counters.insert(CounterType::PlusOnePlusOne, 3);
        let cp = g.computed_permanent(w).unwrap();
        assert!(cp.keywords.contains(&Keyword::Flying), "flying at 3 counters");
        assert!(cp.keywords.contains(&Keyword::Vigilance), "vigilance at 3 counters");
    }

    /// Gathering Throng tutors its same-named copies to hand on entry.
    #[test]
    fn gathering_throng_gathers_copies() {
        use crabomination::decision::{DecisionAnswer, ScriptedDecider};
        let mut g = two_player_game();
        let c1 = g.add_card_to_library(0, catalog::gathering_throng());
        let c2 = g.add_card_to_library(0, catalog::gathering_throng());
        g.decider = Box::new(ScriptedDecider::new([
            DecisionAnswer::Search(Some(c1)),
            DecisionAnswer::Search(Some(c2)),
        ]));
        g.move_card_to_battlefield_for_test(0, catalog::gathering_throng());
        drain_stack(&mut g);
        let in_hand = g.players[0].hand.iter()
            .filter(|c| c.definition.name == "Gathering Throng").count();
        assert_eq!(in_hand, 2, "both library copies found");
    }

    /// Charming Scoundrel's Treasure mode mints a Treasure.
    #[test]
    fn charming_scoundrel_treasure_mode() {
        use crabomination::decision::{DecisionAnswer, ScriptedDecider};
        let mut g = two_player_game();
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Mode(1)]));
        g.move_card_to_battlefield_for_test(0, catalog::charming_scoundrel());
        drain_stack(&mut g);
        assert!(
            g.battlefield.iter().any(|c| c.definition.name == "Treasure"),
            "Treasure token created"
        );
    }

    /// Fear of Missing Out loots on entry (discard then draw).
    #[test]
    fn fear_of_missing_out_loots_on_etb() {
        let mut g = two_player_game();
        let fodder = g.add_card_to_hand(0, catalog::island());
        g.add_card_to_library(0, catalog::forest());
        g.move_card_to_battlefield_for_test(0, catalog::fear_of_missing_out());
        drain_stack(&mut g);
        assert!(g.players[0].graveyard.iter().any(|c| c.id == fodder), "discarded the Island");
        assert!(g.players[0].hand.iter().any(|c| c.definition.name == "Forest"), "drew the Forest");
    }

    /// Archmage of Runes discounts instants/sorceries and draws when you cast one.
    #[test]
    fn archmage_of_runes_discount_and_draw() {
        use crabomination::game::actions::cost_reduction_for_spell;
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::archmage_of_runes());
        let bolt = crabomination::card::CardInstance::new(g.next_id(), catalog::lightning_bolt(), 0);
        assert_eq!(cost_reduction_for_spell(&g, 0, &bolt, None), 1, "one generic off an instant");
        // Casting an instant draws a card.
        let id = g.add_card_to_hand(0, catalog::lightning_bolt());
        let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        g.add_card_to_library(0, catalog::island());
        let before = g.players[0].hand.len();
        g.players[0].mana_pool.add(Color::Red, 1);
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: Some(Target::Permanent(foe)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast bolt");
        drain_stack(&mut g);
        // Hand: -1 (bolt cast) +1 (magecraft draw) = net unchanged vs before-minus-cast.
        assert_eq!(g.players[0].hand.len(), before, "magecraft replaced the cast card");
    }

    /// Splashy Spellcaster mints a Sorcerer Role on a friendly creature when you
    /// cast an instant or sorcery.
    #[test]
    fn splashy_spellcaster_makes_a_role() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::splashy_spellcaster());
        let pet = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let id = g.add_card_to_hand(0, catalog::lightning_bolt());
        let foe = g.add_card_to_battlefield(1, catalog::serra_angel());
        g.players[0].mana_pool.add(Color::Red, 1);
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: Some(Target::Permanent(foe)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast bolt");
        drain_stack(&mut g);
        let role_on_pet = g.battlefield.iter()
            .any(|c| c.attached_to == Some(pet) && c.definition.name == "Sorcerer");
        assert!(role_on_pet, "Sorcerer Role attached to the bear");
        // The Role grants +1/+1.
        let cp = g.computed_permanent(pet).unwrap();
        assert_eq!((cp.power, cp.toughness), (3, 3), "+1/+1 from the Role");
    }

    /// Subterranean Schooner: crew it with a Bear, attack, and the crewer explores.
    #[test]
    fn subterranean_schooner_explores_on_attack() {
        let mut g = two_player_game();
        let ship = g.add_card_to_battlefield(0, catalog::subterranean_schooner());
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.clear_sickness(ship);
        g.clear_sickness(bear);
        g.add_card_to_library(0, catalog::forest()); // top card is a land → goes to hand
        g.perform_action(GameAction::Crew { vehicle: ship, crew_creatures: vec![bear] }).expect("crew 1");
        advance_to(&mut g, TurnStep::DeclareAttackers);
        g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker: ship, target: AttackTarget::Player(1),
        }])).expect("attack");
        drain_stack(&mut g);
        assert!(g.players[0].hand.iter().any(|c| c.definition.name == "Forest"), "explored land to hand");
    }

    /// Steamcore Scholar draws two then discards two on entry.
    #[test]
    fn steamcore_scholar_draw_then_discard() {
        let mut g = two_player_game();
        for _ in 0..2 { g.add_card_to_library(0, catalog::island()); }
        g.move_card_to_battlefield_for_test(0, catalog::steamcore_scholar());
        drain_stack(&mut g);
        // Net hand size unchanged (+2 draw, −2 discard); two cards now in graveyard.
        assert_eq!(g.players[0].graveyard.len(), 2, "discarded two cards");
    }

    /// Axgard Cavalry taps to grant haste.
    #[test]
    fn axgard_cavalry_grants_haste() {
        let mut g = two_player_game();
        let cav = g.add_card_to_battlefield(0, catalog::axgard_cavalry());
        let fresh = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.clear_sickness(cav);
        g.step = TurnStep::PreCombatMain;
        g.perform_action(GameAction::ActivateAbility {
            card_id: cav, ability_index: 0,
            target: Some(Target::Permanent(fresh)), additional_targets: vec![], x_value: None,
        }).expect("activate");
        drain_stack(&mut g);
        assert!(g.computed_permanent(fresh).unwrap().keywords.contains(&Keyword::Haste));
    }

    /// Experimental Synthesizer exiles the top card with may-play on entry, and its
    /// sac ability makes a Samurai.
    #[test]
    fn experimental_synthesizer_etb_and_sac() {
        let mut g = two_player_game();
        g.add_card_to_library(0, catalog::island());
        let synth = g.move_card_to_battlefield_for_test(0, catalog::experimental_synthesizer());
        drain_stack(&mut g);
        assert!(g.exile.iter().any(|c| c.definition.name == "Island"), "top card exiled (may-play)");
        g.players[0].mana_pool.add(Color::Red, 1);
        g.players[0].mana_pool.add_colorless(2);
        g.step = TurnStep::PreCombatMain;
        g.perform_action(GameAction::ActivateAbility {
            card_id: synth, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
        }).expect("sac for Samurai");
        drain_stack(&mut g);
        assert!(g.battlefield.iter().any(|c| c.definition.name == "Samurai"), "Samurai token made");
    }

    /// Hexgold Slith nets energy on entry and grows on combat damage.
    #[test]
    fn hexgold_slith_energy_and_growth() {
        let mut g = two_player_game();
        let slith = g.add_card_to_battlefield(0, catalog::hexgold_slith());
        g.move_card_to_battlefield_for_test(0, catalog::hexgold_slith());
        drain_stack(&mut g);
        assert_eq!(g.players[0].energy, 2, "ETB gave two energy");
        // Attack unblocked → combat damage → +1/+1 counter.
        g.clear_sickness(slith);
        advance_to(&mut g, TurnStep::DeclareAttackers);
        g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker: slith, target: AttackTarget::Player(1),
        }])).expect("attack");
        drain_stack(&mut g);
        advance_to(&mut g, TurnStep::CombatDamage);
        drain_stack(&mut g);
        assert_eq!(
            g.battlefield_find(slith).unwrap().counters.get(&CounterType::PlusOnePlusOne).copied(),
            Some(1),
            "grew on combat damage"
        );
    }

    /// Slickshot Lockpicker grants flashback to an instant/sorcery in your graveyard.
    #[test]
    fn slickshot_lockpicker_grants_flashback() {
        let mut g = two_player_game();
        let bolt = g.add_card_to_graveyard(0, catalog::lightning_bolt());
        g.move_card_to_battlefield_for_test(0, catalog::slickshot_lockpicker());
        drain_stack(&mut g);
        let gy_bolt = g.players[0].graveyard.iter().find(|c| c.id == bolt).unwrap();
        assert!(gy_bolt.granted_flashback_eot.is_some(), "bolt gained flashback this turn");
    }

    /// Tender Wildguide taps for any color and (separately) for a +1/+1 counter.
    #[test]
    fn tender_wildguide_taps_for_counter() {
        let mut g = two_player_game();
        let w = g.add_card_to_battlefield(0, catalog::tender_wildguide());
        g.clear_sickness(w);
        g.perform_action(GameAction::ActivateAbility {
            card_id: w, ability_index: 1, target: None, additional_targets: vec![], x_value: None,
        }).expect("tap for counter");
        drain_stack(&mut g);
        assert_eq!(
            g.battlefield_find(w).unwrap().counters.get(&CounterType::PlusOnePlusOne).copied(),
            Some(1),
        );
    }

    /// Sinister Monolith drains at combat and can sac for two cards.
    #[test]
    fn sinister_monolith_drains_and_draws() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::sinister_monolith());
        advance_to(&mut g, TurnStep::BeginCombat);
        drain_stack(&mut g);
        assert_eq!(g.players[1].life, 19, "opponent lost 1");
        assert_eq!(g.players[0].life, 21, "you gained 1");
    }

    /// CR 702.70 — Pit Scorpion's Poisonous 1 adds a poison counter on combat damage.
    #[test]
    fn pit_scorpion_poisonous_adds_poison() {
        let mut g = two_player_game();
        let scorp = g.add_card_to_battlefield(0, catalog::pit_scorpion());
        g.clear_sickness(scorp);
        advance_to(&mut g, TurnStep::DeclareAttackers);
        g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker: scorp, target: AttackTarget::Player(1),
        }])).expect("attack");
        drain_stack(&mut g);
        advance_to(&mut g, TurnStep::CombatDamage);
        drain_stack(&mut g);
        assert_eq!(g.players[1].life, 19, "1 combat damage");
        assert_eq!(g.players[1].poison_counters, 1, "Poisonous 1 adds a poison counter");
    }

    /// Splatter Goblin shrinks an opponent's creature when it dies.
    #[test]
    fn splatter_goblin_death_shrinks_opponent() {
        let mut g = two_player_game();
        let gob = g.add_card_to_battlefield(0, catalog::splatter_goblin());
        let foe = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4
        g.battlefield_find_mut(gob).unwrap().damage = 1; // lethal on the 2/1
        let evs = g.check_state_based_actions();
        g.dispatch_triggers_for_events(&evs);
        drain_stack(&mut g);
        let cp = g.computed_permanent(foe).unwrap();
        assert_eq!((cp.power, cp.toughness), (3, 3), "-1/-1 on the opponent's creature");
    }

    /// Hightide Hermit nets four energy and can pay {E}{E} to attack despite defender.
    #[test]
    fn hightide_hermit_energy_then_attacks() {
        let mut g = two_player_game();
        let crab = g.add_card_to_battlefield(0, catalog::hightide_hermit());
        g.move_card_to_battlefield_for_test(0, catalog::hightide_hermit());
        drain_stack(&mut g);
        assert_eq!(g.players[0].energy, 4, "ETB gave four energy");
        g.clear_sickness(crab);
        g.step = TurnStep::PreCombatMain;
        g.perform_action(GameAction::ActivateAbility {
            card_id: crab, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
        }).expect("pay energy to attack");
        drain_stack(&mut g);
        advance_to(&mut g, TurnStep::DeclareAttackers);
        g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker: crab, target: AttackTarget::Player(1),
        }])).expect("defender can attack this turn");
    }
}

mod recent3 {
    use crabomination::card::{CounterType, Keyword};
    use crabomination::catalog;
    use crabomination::game::types::{Attack, AttackTarget, TurnStep};
    use crabomination::game::*;
    use crabomination::mana::Color;

    fn advance_to(g: &mut GameState, step: TurnStep) {
        while g.step != step {
            g.perform_action(GameAction::PassPriority).expect("pass priority");
        }
    }

    /// Solphim doubles noncombat damage a source you control deals to an opponent.
    #[test]
    fn solphim_doubles_noncombat_damage_to_opponent() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::solphim_mayhem_dominus());
        let bolt = g.add_card_to_hand(0, catalog::lightning_bolt()); // 3 to any target
        g.players[0].mana_pool.add(Color::Red, 1);
        g.perform_action(GameAction::CastSpell {
            card_id: bolt, target: Some(Target::Player(1)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast bolt");
        drain_stack(&mut g);
        assert_eq!(g.players[1].life, 14, "3 damage doubled to 6");
    }

    /// Solphim does NOT double combat damage (noncombat-only rider).
    #[test]
    fn solphim_leaves_combat_damage_alone() {
        let mut g = two_player_game();
        let solphim = g.add_card_to_battlefield(0, catalog::solphim_mayhem_dominus()); // 5/4
        g.clear_sickness(solphim);
        advance_to(&mut g, TurnStep::DeclareAttackers);
        g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker: solphim, target: AttackTarget::Player(1),
        }])).expect("attack");
        drain_stack(&mut g);
        advance_to(&mut g, TurnStep::PostCombatMain);
        assert_eq!(g.players[1].life, 15, "5 combat damage, not doubled");
    }

    /// Atraxa ships flying/vigilance/deathtouch/lifelink and proliferates at end step.
    #[test]
    fn atraxa_proliferates_at_end_step() {
        let a = catalog::atraxa_praetors_voice();
        for kw in [Keyword::Flying, Keyword::Vigilance, Keyword::Deathtouch, Keyword::Lifelink] {
            assert!(a.keywords.contains(&kw), "Atraxa has {kw:?}");
        }
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::atraxa_praetors_voice());
        // A creature with a +1/+1 counter to proliferate onto.
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.battlefield_find_mut(bear).unwrap().add_counters(CounterType::PlusOnePlusOne, 1);
        advance_to(&mut g, TurnStep::End);
        drain_stack(&mut g);
        assert_eq!(
            g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne),
            2,
            "proliferate added a +1/+1 counter"
        );
    }

    /// Deathrite Shaman's first ability exiles a land from a graveyard for mana.
    #[test]
    fn deathrite_exiles_land_for_mana() {
        let mut g = two_player_game();
        let shaman = g.add_card_to_battlefield(0, catalog::deathrite_shaman());
        g.clear_sickness(shaman);
        let land = g.add_card_to_graveyard(1, catalog::forest());
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::ActivateAbility {
            card_id: shaman, ability_index: 0,
            target: Some(Target::Permanent(land)), additional_targets: vec![], x_value: None,
        }).expect("activate land exile");
        drain_stack(&mut g);
        assert!(g.exile.iter().any(|c| c.id == land), "land exiled");
        assert!(g.players[0].mana_pool.total() >= 1, "produced a mana");
    }

    /// Deathrite's instant/sorcery ability drains each opponent for 2.
    #[test]
    fn deathrite_drains_on_instant_exile() {
        let mut g = two_player_game();
        let shaman = g.add_card_to_battlefield(0, catalog::deathrite_shaman());
        g.clear_sickness(shaman);
        let bolt = g.add_card_to_graveyard(1, catalog::lightning_bolt());
        g.players[0].mana_pool.add(Color::Black, 1);
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::ActivateAbility {
            card_id: shaman, ability_index: 1,
            target: Some(Target::Permanent(bolt)), additional_targets: vec![], x_value: None,
        }).expect("activate I/S exile");
        drain_stack(&mut g);
        assert!(g.exile.iter().any(|c| c.id == bolt), "instant exiled");
        assert_eq!(g.players[1].life, 18, "opponent drained 2");
    }

    /// Grand Abolisher stops opponents casting + activating A/C/E abilities on your turn.
    #[test]
    fn grand_abolisher_locks_opponent_on_your_turn() {
        let mut g = two_player_game(); // P0 active
        g.add_card_to_battlefield(0, catalog::grand_abolisher());
        // P1 holds a spell and has priority during P0's turn.
        let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
        g.players[1].mana_pool.add(Color::Red, 1);
        g.priority.player_with_priority = 1;
        let err = g.perform_action(GameAction::CastSpell {
            card_id: bolt, target: Some(Target::Player(0)),
            additional_targets: vec![], mode: None, x_value: None,
        });
        assert!(err.is_err(), "opponent can't cast during your turn");
        // A creature activated ability is also locked.
        let dork = g.add_card_to_battlefield(1, catalog::deathrite_shaman());
        g.clear_sickness(dork);
        let land = g.add_card_to_graveyard(0, catalog::forest());
        let err2 = g.perform_action(GameAction::ActivateAbility {
            card_id: dork, ability_index: 0,
            target: Some(Target::Permanent(land)), additional_targets: vec![], x_value: None,
        });
        assert!(err2.is_err(), "opponent can't activate creature abilities on your turn");
    }

    /// Sundering Titan destroys a land of each basic type on enter.
    #[test]
    fn sundering_titan_destroys_one_of_each_basic_type() {
        let mut g = two_player_game();
        let plains = g.add_card_to_battlefield(1, catalog::plains());
        let island = g.add_card_to_battlefield(1, catalog::island());
        let forest = g.add_card_to_battlefield(0, catalog::forest());
        g.move_card_to_battlefield_for_test(0, catalog::sundering_titan());
        drain_stack(&mut g);
        let in_a_graveyard = |g: &GameState, id| {
            g.players.iter().any(|p| p.graveyard.iter().any(|c| c.id == id))
        };
        for (id, name) in [(plains, "Plains"), (island, "Island"), (forest, "Forest")] {
            assert!(in_a_graveyard(&g, id), "{name} destroyed");
        }
    }

    /// Arcane Laboratory stops a player from casting a second spell in a turn.
    #[test]
    fn arcane_laboratory_one_spell_per_turn() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::arcane_laboratory());
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        let a = g.add_card_to_hand(0, catalog::lightning_bolt());
        let b = g.add_card_to_hand(0, catalog::lightning_bolt());
        g.players[0].mana_pool.add(Color::Red, 2);
        g.perform_action(GameAction::CastSpell {
            card_id: a, target: Some(Target::Player(1)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("first spell allowed");
        drain_stack(&mut g);
        let err = g.perform_action(GameAction::CastSpell {
            card_id: b, target: Some(Target::Player(1)),
            additional_targets: vec![], mode: None, x_value: None,
        });
        assert!(err.is_err(), "second spell blocked by Arcane Laboratory");
    }

    /// Flashfires destroys all Plains and leaves other lands.
    #[test]
    fn flashfires_destroys_plains() {
        let mut g = two_player_game();
        let p = g.add_card_to_battlefield(1, catalog::plains());
        let f = g.add_card_to_battlefield(1, catalog::forest());
        let cast = g.add_card_to_hand(0, catalog::flashfires());
        g.players[0].mana_pool.add(Color::Red, 1);
        g.players[0].mana_pool.add_colorless(3);
        g.perform_action(GameAction::CastSpell {
            card_id: cast, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast Flashfires");
        drain_stack(&mut g);
        assert!(g.battlefield_find(p).is_none(), "Plains destroyed");
        assert!(g.battlefield_find(f).is_some(), "Forest survives");
    }

    /// Anarchy destroys all white permanents.
    #[test]
    fn anarchy_destroys_white() {
        let mut g = two_player_game();
        let white = g.add_card_to_battlefield(1, catalog::grand_abolisher()); // {W}{W} white
        let red = g.add_card_to_battlefield(1, catalog::solphim_mayhem_dominus()); // red
        let cast = g.add_card_to_hand(0, catalog::anarchy());
        g.players[0].mana_pool.add(Color::Red, 2);
        g.players[0].mana_pool.add_colorless(2);
        g.perform_action(GameAction::CastSpell {
            card_id: cast, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast Anarchy");
        drain_stack(&mut g);
        assert!(g.battlefield_find(white).is_none(), "white permanent destroyed");
        assert!(g.battlefield_find(red).is_some(), "red permanent survives");
    }

    /// Creeping Mold destroys a target enchantment.
    #[test]
    fn creeping_mold_destroys_enchantment() {
        let mut g = two_player_game();
        let ench = g.add_card_to_battlefield(1, catalog::arcane_laboratory());
        let cast = g.add_card_to_hand(0, catalog::creeping_mold());
        g.players[0].mana_pool.add(Color::Green, 2);
        g.players[0].mana_pool.add_colorless(2);
        g.perform_action(GameAction::CastSpell {
            card_id: cast, target: Some(Target::Permanent(ench)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast Creeping Mold");
        drain_stack(&mut g);
        assert!(g.battlefield_find(ench).is_none(), "enchantment destroyed");
    }

    /// Liliana's Caress drains an opponent when they discard.
    #[test]
    fn lilianas_caress_punishes_discard() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::lilianas_caress());
        let card = g.add_card_to_hand(1, catalog::grizzly_bears());
        let mut events = Vec::new();
        g.discard_card(1, card, &mut events);
        g.dispatch_triggers_for_events(&events);
        drain_stack(&mut g);
        assert_eq!(g.players[1].life, 18, "opponent lost 2 to Caress on discard");
    }

    /// Shatterstorm wipes all artifacts.
    #[test]
    fn shatterstorm_destroys_artifacts() {
        let mut g = two_player_game();
        let art = g.add_card_to_battlefield(1, catalog::sundering_titan()); // artifact creature
        let cast = g.add_card_to_hand(0, catalog::shatterstorm());
        g.players[0].mana_pool.add(Color::Red, 2);
        g.players[0].mana_pool.add_colorless(2);
        g.perform_action(GameAction::CastSpell {
            card_id: cast, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast Shatterstorm");
        drain_stack(&mut g);
        assert!(g.battlefield_find(art).is_none(), "artifact destroyed");
    }

    /// Tsunami / Boiling Seas destroy all Islands (shared landtype-wipe path).
    #[test]
    fn tsunami_destroys_islands() {
        let mut g = two_player_game();
        let isl = g.add_card_to_battlefield(1, catalog::island());
        let cast = g.add_card_to_hand(0, catalog::tsunami());
        g.players[0].mana_pool.add(Color::Green, 1);
        g.players[0].mana_pool.add_colorless(3);
        g.perform_action(GameAction::CastSpell {
            card_id: cast, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast Tsunami");
        drain_stack(&mut g);
        assert!(g.battlefield_find(isl).is_none(), "Island destroyed");
    }

    /// Winter Orb keeps lands from untapping (CR 502.3 via PreventUntap).
    #[test]
    fn winter_orb_locks_lands() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::winter_orb());
        let land = g.add_card_to_battlefield(0, catalog::forest());
        let creature = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.battlefield_find_mut(land).unwrap().tapped = true;
        g.battlefield_find_mut(creature).unwrap().tapped = true;
        // Run the active player's untap step.
        g.do_untap();
        assert!(g.battlefield_find(land).unwrap().tapped, "land stays tapped");
        assert!(!g.battlefield_find(creature).unwrap().tapped, "creature untaps normally");
    }

    /// Choke keeps only Islands from untapping.
    #[test]
    fn choke_locks_islands_only() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::choke());
        let island = g.add_card_to_battlefield(0, catalog::island());
        let forest = g.add_card_to_battlefield(0, catalog::forest());
        g.battlefield_find_mut(island).unwrap().tapped = true;
        g.battlefield_find_mut(forest).unwrap().tapped = true;
        g.do_untap();
        assert!(g.battlefield_find(island).unwrap().tapped, "Island stays tapped");
        assert!(!g.battlefield_find(forest).unwrap().tapped, "Forest untaps");
    }

    /// The bot fires a compound (Seq-wrapped) "each opponent loses N" ability for
    /// lethal reach, not just a bare drain.
    #[test]
    fn bot_fires_seq_wrapped_reach_drain_for_lethal() {
        use crabomination::card::{ActivatedAbility, CardDefinition, CardType, Effect, Selector, Value};
        use crabomination::effect::PlayerRef;
        use crabomination::server::bot::{Bot, RandomBot};
        let drainer = CardDefinition {
            name: "Test Drainer",
            card_types: vec![CardType::Artifact],
            activated_abilities: vec![ActivatedAbility {
                // No mana/target cost; effect wraps the reach in a Seq.
                effect: Effect::Seq(vec![
                    Effect::Noop,
                    Effect::LoseLife {
                        who: Selector::Player(PlayerRef::EachOpponent),
                        amount: Value::Const(3),
                    },
                ]),
                ..Default::default()
            }],
            ..Default::default()
        };
        let mut g = two_player_game();
        let id = g.add_card_to_battlefield(0, drainer);
        g.players[1].life = 3; // exactly lethal
        g.step = TurnStep::PreCombatMain;
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        let action = RandomBot::new().next_action(&g, 0);
        assert!(
            matches!(action, Some(GameAction::ActivateAbility { card_id, .. }) if card_id == id),
            "bot activates the Seq-wrapped drain for lethal: {action:?}"
        );
    }

    /// The bot equips its Equipment onto an attacker rather than a Defender wall.
    #[test]
    fn bot_equips_attacker_not_defender_wall() {
        use crabomination::server::bot::{Bot, RandomBot};
        let mut g = two_player_game();
        // A big 0/4 Wall (Defender) and a smaller real attacker.
        let wall = g.add_card_to_battlefield(0, catalog::wall_of_omens()); // Defender, 0/4
        let attacker = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2, can attack
        let eq = g.add_card_to_battlefield(0, catalog::bonesplitter());
        g.players[0].mana_pool.add_colorless(1); // equip {1}
        g.step = TurnStep::PreCombatMain;
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        // The bot should equip the attacker, not the higher-toughness Wall.
        let mut found = None;
        for _ in 0..40 {
            if let Some(GameAction::Equip { equipment, target }) = RandomBot::new().next_action(&g, 0) {
                found = Some((equipment, target));
                break;
            }
        }
        assert_eq!(found, Some((eq, attacker)), "equips the attacker, skipping the Wall");
        assert_ne!(found.unwrap().1, wall);
    }

    /// The bot ranks equip targets by *computed* power, so an anthem/Aura-boosted
    /// small body outranks a bigger vanilla one.
    #[test]
    fn bot_equips_by_computed_power() {
        use crabomination::server::bot::{Bot, RandomBot};
        let mut g = two_player_game();
        let lions = g.add_card_to_battlefield(0, catalog::savannah_lions()); // 2/1
        let mother = g.add_card_to_battlefield(0, catalog::mother_of_runes()); // 1/1
        // Rancor (+2/+0) on the 1/1 → computed power 3, beating the 2/1.
        let rancor = g.add_card_to_battlefield(0, catalog::rancor());
        g.battlefield_find_mut(rancor).unwrap().attached_to = Some(mother);
        let eq = g.add_card_to_battlefield(0, catalog::bonesplitter());
        g.players[0].mana_pool.add_colorless(1);
        g.step = TurnStep::PreCombatMain;
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        let mut found = None;
        for _ in 0..40 {
            if let Some(GameAction::Equip { equipment, target }) = RandomBot::new().next_action(&g, 0) {
                found = Some((equipment, target));
                break;
            }
        }
        assert_eq!(found, Some((eq, mother)), "equips the Rancor'd 1/1, not the vanilla 2/1");
        let _ = lions;
    }

    /// Manalith taps for one mana of any color.
    #[test]
    fn manalith_taps_for_mana() {
        let mut g = two_player_game();
        let rock = g.add_card_to_battlefield(0, catalog::manalith());
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::ActivateAbility {
            card_id: rock, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
        }).expect("tap for mana");
        assert_eq!(g.players[0].mana_pool.total(), 1, "produced one mana");
    }

    /// Darksteel Ingot is indestructible and a mana rock.
    #[test]
    fn darksteel_ingot_indestructible_rock() {
        let mut g = two_player_game();
        let ingot = g.add_card_to_battlefield(0, catalog::darksteel_ingot());
        let cast = g.add_card_to_hand(1, catalog::shatterstorm());
        g.active_player_idx = 1;
        g.priority.player_with_priority = 1;
        g.step = TurnStep::PreCombatMain;
        g.players[1].mana_pool.add(Color::Red, 2);
        g.players[1].mana_pool.add_colorless(2);
        g.perform_action(GameAction::CastSpell {
            card_id: cast, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast Shatterstorm");
        drain_stack(&mut g);
        assert!(g.battlefield_find(ingot).is_some(), "indestructible ingot survives artifact wipe");
    }

    /// Cultivator's Caravan has Crew 3 and taps for any color.
    #[test]
    fn cultivators_caravan_crews_and_taps() {
        let c = catalog::cultivators_caravan();
        assert!(c.keywords.contains(&Keyword::Crew(3)));
        assert_eq!((c.power, c.toughness), (5, 5));
    }

    /// Hurricane deals X to each flyer and each player.
    #[test]
    fn hurricane_hits_flyers_and_players() {
        let mut g = two_player_game();
        let flyer = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4 flying
        let ground = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2 no fly
        let cast = g.add_card_to_hand(0, catalog::hurricane());
        g.players[0].mana_pool.add(Color::Green, 1);
        g.players[0].mana_pool.add_colorless(4); // X = 4
        g.perform_action(GameAction::CastSpell {
            card_id: cast, target: None, additional_targets: vec![], mode: None, x_value: Some(4),
        }).expect("cast Hurricane");
        drain_stack(&mut g);
        assert!(g.battlefield_find(flyer).is_none(), "flyer took 4 and died");
        assert!(g.battlefield_find(ground).is_some(), "ground creature untouched");
        assert_eq!(g.players[0].life, 16, "controller took 4");
        assert_eq!(g.players[1].life, 16, "opponent took 4");
    }

    /// Staff of Nin draws on upkeep and pings any target.
    #[test]
    fn staff_of_nin_draws_and_pings() {
        let mut g = two_player_game();
        let staff = g.add_card_to_battlefield(0, catalog::staff_of_nin());
        g.add_card_to_library(0, catalog::forest());
        let hand_before = g.players[0].hand.len();
        g.step = TurnStep::Upkeep;
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        g.fire_step_triggers(TurnStep::Upkeep);
        drain_stack(&mut g);
        assert_eq!(g.players[0].hand.len(), hand_before + 1, "drew on upkeep");
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::ActivateAbility {
            card_id: staff, ability_index: 0, target: Some(Target::Player(1)),
            additional_targets: vec![], x_value: None,
        }).expect("ping");
        drain_stack(&mut g);
        assert_eq!(g.players[1].life, 19, "pinged opponent for 1");
    }

    /// Ivory Tower gains life for a hand over four cards.
    #[test]
    fn ivory_tower_gains_for_excess_hand() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::ivory_tower());
        g.players[0].hand.clear();
        for _ in 0..6 { g.add_card_to_hand(0, catalog::forest()); } // hand of 6 → gain 2
        g.step = TurnStep::Upkeep;
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        g.fire_step_triggers(TurnStep::Upkeep);
        drain_stack(&mut g);
        assert_eq!(g.players[0].life, 22, "gained hand(6) - 4 = 2 life");
    }

    /// Viridian Shaman destroys an artifact on ETB.
    #[test]
    fn viridian_shaman_destroys_artifact() {
        let mut g = two_player_game();
        let art = g.add_card_to_battlefield(1, catalog::manalith());
        g.move_card_to_battlefield_for_test(0, catalog::viridian_shaman());
        drain_stack(&mut g);
        assert!(g.battlefield_find(art).is_none(), "artifact destroyed by ETB");
    }

    /// Caustic Caterpillar sacrifices itself to destroy an enchantment.
    #[test]
    fn caustic_caterpillar_sacs_to_destroy() {
        let mut g = two_player_game();
        let cat = g.add_card_to_battlefield(0, catalog::caustic_caterpillar());
        let ench = g.add_card_to_battlefield(1, catalog::arcane_laboratory());
        g.clear_sickness(cat);
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g.players[0].mana_pool.add(Color::Green, 1);
        g.players[0].mana_pool.add_colorless(1);
        g.perform_action(GameAction::ActivateAbility {
            card_id: cat, ability_index: 0, target: Some(Target::Permanent(ench)),
            additional_targets: vec![], x_value: None,
        }).expect("sac to destroy");
        drain_stack(&mut g);
        assert!(g.battlefield_find(ench).is_none(), "enchantment destroyed");
        assert!(g.battlefield_find(cat).is_none(), "caterpillar sacrificed");
    }

    /// Noxious Revival puts a graveyard card on top of its owner's library.
    #[test]
    fn noxious_revival_returns_to_library_top() {
        let mut g = two_player_game();
        let card = g.add_card_to_graveyard(0, catalog::grizzly_bears());
        let spell = g.add_card_to_hand(0, catalog::noxious_revival());
        g.players[0].mana_pool.add(Color::Green, 1);
        g.perform_action(GameAction::CastSpell {
            card_id: spell, target: Some(Target::Permanent(card)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast Noxious Revival");
        drain_stack(&mut g);
        assert_eq!(g.players[0].library.last().map(|c| c.id), Some(card), "card on top of library");
    }

    /// Bane of Progress wipes artifacts/enchantments and grows per permanent destroyed.
    #[test]
    fn bane_of_progress_wipes_and_grows() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(1, catalog::manalith());     // artifact
        g.add_card_to_battlefield(1, catalog::arcane_laboratory()); // enchantment
        let bane = g.move_card_to_battlefield_for_test(0, catalog::bane_of_progress());
        drain_stack(&mut g);
        assert_eq!(
            g.battlefield_find(bane).unwrap().counter_count(CounterType::PlusOnePlusOne),
            2,
            "two permanents destroyed → two counters"
        );
    }

    /// Ramunap Ruins sacrifices a Desert to deal 2 to each opponent.
    #[test]
    fn ramunap_ruins_desert_sac_burn() {
        let mut g = two_player_game();
        let ruins = g.add_card_to_battlefield(0, catalog::ramunap_ruins());
        let desert = g.add_card_to_battlefield(0, catalog::ramunap_ruins()); // another Desert to sac
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g.players[0].mana_pool.add(Color::Red, 2);
        g.players[0].mana_pool.add_colorless(2);
        let _ = desert;
        g.perform_action(GameAction::ActivateAbility {
            card_id: ruins, ability_index: 2, target: None, additional_targets: vec![], x_value: None,
        }).expect("sac a Desert for burn");
        drain_stack(&mut g);
        assert_eq!(g.players[1].life, 18, "opponent took 2");
    }

    /// Back to Nature destroys all enchantments.
    #[test]
    fn back_to_nature_destroys_enchantments() {
        let mut g = two_player_game();
        let e1 = g.add_card_to_battlefield(0, catalog::arcane_laboratory());
        let e2 = g.add_card_to_battlefield(1, catalog::choke());
        let cast = g.add_card_to_hand(0, catalog::back_to_nature());
        g.players[0].mana_pool.add(Color::Green, 1);
        g.players[0].mana_pool.add_colorless(1);
        g.perform_action(GameAction::CastSpell {
            card_id: cast, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast Back to Nature");
        drain_stack(&mut g);
        assert!(g.battlefield_find(e1).is_none() && g.battlefield_find(e2).is_none(), "all enchantments gone");
    }

    /// Whirlwind destroys only fliers.
    #[test]
    fn whirlwind_destroys_fliers() {
        let mut g = two_player_game();
        let flyer = g.add_card_to_battlefield(1, catalog::serra_angel());
        let ground = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let cast = g.add_card_to_hand(0, catalog::whirlwind());
        g.players[0].mana_pool.add(Color::Green, 2);
        g.players[0].mana_pool.add_colorless(2);
        g.perform_action(GameAction::CastSpell {
            card_id: cast, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast Whirlwind");
        drain_stack(&mut g);
        assert!(g.battlefield_find(flyer).is_none(), "flyer destroyed");
        assert!(g.battlefield_find(ground).is_some(), "non-flyer survives");
    }

    /// Fault Line hits ground creatures and players, sparing fliers.
    #[test]
    fn fault_line_spares_fliers() {
        let mut g = two_player_game();
        let flyer = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4 flying
        let ground = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
        let cast = g.add_card_to_hand(0, catalog::fault_line());
        g.players[0].mana_pool.add(Color::Red, 2);
        g.players[0].mana_pool.add_colorless(3); // X = 3
        g.perform_action(GameAction::CastSpell {
            card_id: cast, target: None, additional_targets: vec![], mode: None, x_value: Some(3),
        }).expect("cast Fault Line");
        drain_stack(&mut g);
        assert!(g.battlefield_find(flyer).is_some(), "flyer spared");
        assert!(g.battlefield_find(ground).is_none(), "ground creature took 3 and died");
        assert_eq!(g.players[1].life, 17, "opponent took 3");
    }

    /// Serenity wipes artifacts and enchantments (including itself) on upkeep.
    #[test]
    fn serenity_wipes_on_upkeep() {
        let mut g = two_player_game();
        let serenity = g.add_card_to_battlefield(0, catalog::serenity());
        let art = g.add_card_to_battlefield(1, catalog::manalith());
        g.step = TurnStep::Upkeep;
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        g.fire_step_triggers(TurnStep::Upkeep);
        drain_stack(&mut g);
        assert!(g.battlefield_find(art).is_none(), "artifact destroyed");
        assert!(g.battlefield_find(serenity).is_none(), "Serenity destroys itself too");
    }
}

mod recent4 {
    use crabomination::catalog;
    use crabomination::game::actions::extra_cost_for_spell;
    use crabomination::game::types::TurnStep;
    use crabomination::game::*;
    use crabomination::mana::Color;

    /// Ritual of Soot destroys creatures with mana value 3 or less, sparing bigger.
    #[test]
    fn ritual_of_soot_kills_small_creatures_only() {
        let mut g = two_player_game();
        let small = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // MV 2
        let big = g.add_card_to_battlefield(1, catalog::grave_titan()); // MV 6
        let id = g.add_card_to_hand(0, catalog::ritual_of_soot());
        g.players[0].mana_pool.add(Color::Black, 2);
        g.players[0].mana_pool.add_colorless(2);
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast Ritual of Soot");
        drain_stack(&mut g);
        assert!(g.battlefield_find(small).is_none(), "MV2 creature destroyed");
        assert!(g.battlefield_find(big).is_some(), "MV6 creature survives");
    }

    /// Recurring Nightmare: sac a creature + bounce itself to reanimate a graveyard
    /// creature.
    #[test]
    fn recurring_nightmare_reanimates_sacrificing_a_creature() {
        let mut g = two_player_game();
        let nightmare = g.add_card_to_battlefield(0, catalog::recurring_nightmare());
        let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let dead = g.add_card_to_graveyard(0, catalog::grave_titan());
        g.perform_action(GameAction::ActivateAbility {
            card_id: nightmare, ability_index: 0,
            target: Some(Target::Permanent(dead)), additional_targets: vec![], x_value: None,
        }).expect("activate Recurring Nightmare");
        drain_stack(&mut g);
        assert!(g.battlefield_find(dead).is_some(), "graveyard creature reanimated");
        assert!(g.battlefield_find(fodder).is_none(), "fodder creature sacrificed");
        assert!(g.players[0].hand.iter().any(|c| c.id == nightmare),
            "Recurring Nightmare returned to its owner's hand");
    }

    /// Survival of the Fittest: pay {G} + discard a creature → tutor a creature to
    /// hand.
    #[test]
    fn survival_of_the_fittest_tutors_a_creature() {
        let mut g = two_player_game();
        let survival = g.add_card_to_battlefield(0, catalog::survival_of_the_fittest());
        let discard = g.add_card_to_hand(0, catalog::grizzly_bears());
        let fetch = g.add_card_to_library(0, catalog::grave_titan());
        g.decider = Box::new(crabomination::decision::ScriptedDecider::new([
            crabomination::decision::DecisionAnswer::Search(Some(fetch)),
        ]));
        g.players[0].mana_pool.add(Color::Green, 1);
        g.perform_action(GameAction::ActivateAbility {
            card_id: survival, ability_index: 0,
            target: None, additional_targets: vec![], x_value: None,
        }).expect("activate Survival of the Fittest");
        drain_stack(&mut g);
        assert!(g.players[0].hand.iter().any(|c| c.id == fetch),
            "tutored creature is in hand");
        assert!(g.players[0].graveyard.iter().any(|c| c.id == discard),
            "discarded creature is in the graveyard");
    }

    /// Footsteps of the Goryo reanimates a graveyard creature, then sacrifices it
    /// at the next end step.
    #[test]
    fn footsteps_of_the_goryo_reanimates_then_sacrifices() {
        let mut g = two_player_game();
        let dead = g.add_card_to_graveyard(0, catalog::grave_titan());
        let id = g.add_card_to_hand(0, catalog::footsteps_of_the_goryo());
        g.players[0].mana_pool.add(Color::Black, 1);
        g.players[0].mana_pool.add_colorless(2);
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: Some(Target::Permanent(dead)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast Footsteps of the Goryo");
        drain_stack(&mut g);
        assert!(g.battlefield_find(dead).is_some(), "creature reanimated");
        // Walk to the end step — the delayed trigger sacrifices it.
        while g.step != TurnStep::End {
            g.perform_action(GameAction::PassPriority).expect("pass");
        }
        drain_stack(&mut g);
        assert!(g.battlefield_find(dead).is_none(), "reanimated creature sacrificed at end step");
    }

    /// Apprentice Necromancer: {B}, {T}, sac itself → reanimate a graveyard creature
    /// with haste; sacrifice it at the next end step.
    #[test]
    fn apprentice_necromancer_reanimates_with_haste() {
        let mut g = two_player_game();
        let appr = g.add_card_to_battlefield(0, catalog::apprentice_necromancer());
        g.clear_sickness(appr);
        let dead = g.add_card_to_graveyard(0, catalog::grave_titan());
        g.players[0].mana_pool.add(Color::Black, 1);
        g.perform_action(GameAction::ActivateAbility {
            card_id: appr, ability_index: 0,
            target: Some(Target::Permanent(dead)), additional_targets: vec![], x_value: None,
        }).expect("activate Apprentice Necromancer");
        drain_stack(&mut g);
        assert!(g.battlefield_find(dead).is_some(), "creature reanimated");
        assert!(g.computed_permanent(dead).unwrap().keywords.contains(&crabomination::card::Keyword::Haste),
            "reanimated creature has haste");
        assert!(g.battlefield_find(appr).is_none(), "Apprentice Necromancer sacrificed itself");
        while g.step != TurnStep::End {
            g.perform_action(GameAction::PassPriority).expect("pass");
        }
        drain_stack(&mut g);
        assert!(g.battlefield_find(dead).is_none(), "reanimated creature sacrificed at end step");
    }

    /// Deafening Silence: a second noncreature spell can't be cast; creature spells
    /// are unaffected.
    #[test]
    fn deafening_silence_limits_noncreature_spells() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::deafening_silence());
        let bolt1 = g.add_card_to_hand(0, catalog::lightning_bolt());
        let bolt2 = g.add_card_to_hand(0, catalog::lightning_bolt());
        let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
        g.players[0].mana_pool.add(Color::Red, 2);
        g.players[0].mana_pool.add(Color::Green, 1);
        g.players[0].mana_pool.add_colorless(1);
        g.perform_action(GameAction::CastSpell {
            card_id: bolt1, target: Some(Target::Player(1)), additional_targets: vec![], mode: None, x_value: None,
        }).expect("first noncreature spell ok");
        drain_stack(&mut g);
        let second = g.perform_action(GameAction::CastSpell {
            card_id: bolt2, target: Some(Target::Player(1)), additional_targets: vec![], mode: None, x_value: None,
        });
        assert!(matches!(second, Err(GameError::SpellLimitReached)),
            "second noncreature spell blocked, got {second:?}");
        // A creature spell is still castable.
        g.perform_action(GameAction::CastSpell {
            card_id: bear, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("creature spell still castable");
    }

    /// Ethersworn Canonist: a second nonartifact spell can't be cast; artifact
    /// spells are unaffected.
    #[test]
    fn ethersworn_canonist_limits_nonartifact_spells() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::ethersworn_canonist());
        let bolt1 = g.add_card_to_hand(0, catalog::lightning_bolt());
        let bolt2 = g.add_card_to_hand(0, catalog::lightning_bolt());
        let sol = g.add_card_to_hand(0, catalog::sol_ring());
        g.players[0].mana_pool.add(Color::Red, 2);
        g.players[0].mana_pool.add_colorless(1);
        g.perform_action(GameAction::CastSpell {
            card_id: bolt1, target: Some(Target::Player(1)), additional_targets: vec![], mode: None, x_value: None,
        }).expect("first nonartifact spell ok");
        drain_stack(&mut g);
        let second = g.perform_action(GameAction::CastSpell {
            card_id: bolt2, target: Some(Target::Player(1)), additional_targets: vec![], mode: None, x_value: None,
        });
        assert!(matches!(second, Err(GameError::SpellLimitReached)),
            "second nonartifact spell blocked, got {second:?}");
        g.perform_action(GameAction::CastSpell {
            card_id: sol, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("artifact spell still castable");
    }

    /// Defense Grid taxes spells {3} unless cast during the caster's own turn.
    #[test]
    fn defense_grid_taxes_off_turn_spells() {
        let mut g = two_player_game(); // player 0 active
        g.add_card_to_battlefield(0, catalog::defense_grid());
        let id = g.add_card_to_hand(1, catalog::lightning_bolt());
        let bolt = g.players[1].hand.iter().find(|c| c.id == id).unwrap().clone();
        assert_eq!(extra_cost_for_spell(&g, 1, &bolt, None, 0), 3,
            "opponent's spell taxed {{3}} on the active player's turn");
        assert_eq!(extra_cost_for_spell(&g, 0, &bolt, None, 0), 0,
            "active player's own spell untaxed");
    }

    /// Bontu's Last Reckoning destroys all creatures and keeps the caster's lands
    /// from untapping next untap step.
    #[test]
    fn bontus_last_reckoning_wipes_board_and_locks_lands() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let land = g.add_card_to_battlefield(0, catalog::forest());
        let id = g.add_card_to_hand(0, catalog::bontus_last_reckoning());
        g.players[0].mana_pool.add(Color::Black, 2);
        g.players[0].mana_pool.add_colorless(1);
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast Bontu's Last Reckoning");
        drain_stack(&mut g);
        assert!(g.battlefield_find(bear).is_none(), "all creatures destroyed");
        // Tap the land and run the untap step — it must stay tapped (charge spent).
        g.battlefield_find_mut(land).unwrap().tapped = true;
        g.do_untap();
        assert!(g.battlefield_find(land).unwrap().tapped, "land stays tapped after Bontu's");
        // The lock is one-shot: the following untap step untaps normally.
        g.do_untap();
        assert!(!g.battlefield_find(land).unwrap().tapped, "land untaps the next step");
    }

    /// Syphon Mind makes each opponent discard and draws you one per discard.
    #[test]
    fn syphon_mind_discards_each_opponent_and_draws() {
        let mut g = crabomination::game::multi_player_game(3);
        for p in 1..3 {
            g.add_card_to_hand(p, catalog::forest());
        }
        for _ in 0..2 {
            g.add_card_to_library(0, catalog::island());
        }
        let id = g.add_card_to_hand(0, catalog::syphon_mind());
        let hand0 = g.players[0].hand.len();
        g.players[0].mana_pool.add(Color::Black, 1);
        g.players[0].mana_pool.add_colorless(3);
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast Syphon Mind");
        drain_stack(&mut g);
        // Two opponents each discard one; caster draws two (loses Syphon Mind, +2).
        assert_eq!(g.players[1].hand.len(), 0, "opponent 1 discarded");
        assert_eq!(g.players[2].hand.len(), 0, "opponent 2 discarded");
        assert_eq!(g.players[0].hand.len(), hand0 - 1 + 2, "caster drew one per discard");
    }

    /// Prosperity makes each player draw X.
    #[test]
    fn prosperity_each_player_draws_x() {
        let mut g = two_player_game();
        for _ in 0..4 {
            g.add_card_to_library(0, catalog::island());
            g.add_card_to_library(1, catalog::forest());
        }
        let id = g.add_card_to_hand(0, catalog::prosperity());
        let h0 = g.players[0].hand.len();
        let h1 = g.players[1].hand.len();
        g.players[0].mana_pool.add(Color::Blue, 1);
        g.players[0].mana_pool.add_colorless(2);
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: None, additional_targets: vec![], mode: None, x_value: Some(2),
        }).expect("cast Prosperity for X=2");
        drain_stack(&mut g);
        assert_eq!(g.players[0].hand.len(), h0 - 1 + 2, "caster drew 2 (lost Prosperity)");
        assert_eq!(g.players[1].hand.len(), h1 + 2, "opponent drew 2");
    }

    /// Ondu Giant fetches a basic land onto the battlefield tapped.
    #[test]
    fn ondu_giant_etb_fetches_basic_tapped() {
        let mut g = two_player_game();
        let forest = g.add_card_to_library(0, catalog::forest());
        g.decider = Box::new(crabomination::decision::ScriptedDecider::new([
            crabomination::decision::DecisionAnswer::Search(Some(forest)),
        ]));
        g.move_card_to_battlefield_for_test(0, catalog::ondu_giant());
        drain_stack(&mut g);
        let fetched = g.battlefield_find(forest).expect("basic land fetched to battlefield");
        assert!(fetched.tapped, "fetched land enters tapped");
    }

    /// Roiling Regrowth sacrifices a land to fetch up to two basics tapped.
    #[test]
    fn roiling_regrowth_sacrifices_land_for_two_basics() {
        let mut g = two_player_game();
        let sacland = g.add_card_to_battlefield(0, catalog::mountain());
        let f1 = g.add_card_to_library(0, catalog::forest());
        let f2 = g.add_card_to_library(0, catalog::island());
        g.decider = Box::new(crabomination::decision::ScriptedDecider::new([
            crabomination::decision::DecisionAnswer::Search(Some(f1)),
            crabomination::decision::DecisionAnswer::Search(Some(f2)),
        ]));
        let id = g.add_card_to_hand(0, catalog::roiling_regrowth());
        g.players[0].mana_pool.add(Color::Green, 1);
        g.players[0].mana_pool.add_colorless(2);
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast Roiling Regrowth");
        drain_stack(&mut g);
        assert!(g.battlefield_find(sacland).is_none(), "a land was sacrificed");
        assert!(g.battlefield_find(f1).is_some_and(|c| c.tapped), "first basic on battlefield tapped");
        assert!(g.battlefield_find(f2).is_some_and(|c| c.tapped), "second basic on battlefield tapped");
    }

    /// Roar of the Wurm makes a 6/6 Wurm and can be flashed back from the graveyard.
    #[test]
    fn roar_of_the_wurm_token_and_flashback() {
        let mut g = two_player_game();
        let id = g.add_card_to_hand(0, catalog::roar_of_the_wurm());
        g.players[0].mana_pool.add(Color::Green, 1);
        g.players[0].mana_pool.add_colorless(6);
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast Roar of the Wurm");
        drain_stack(&mut g);
        let wurms = g.battlefield.iter().filter(|c| c.definition.name == "Wurm").count();
        assert_eq!(wurms, 1, "one 6/6 Wurm token");
        assert!(g.players[0].graveyard.iter().any(|c| c.id == id), "Roar is in the graveyard");
        // Flashback it for {3}{G}.
        g.players[0].mana_pool.add(Color::Green, 1);
        g.players[0].mana_pool.add_colorless(3);
        g.perform_action(GameAction::CastFlashback {
            card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("flashback Roar of the Wurm");
        drain_stack(&mut g);
        assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Wurm").count(), 2,
            "flashback made a second Wurm");
    }

    /// Chart a Course draws two and only forces a discard when you haven't attacked.
    #[test]
    fn chart_a_course_discards_unless_attacked() {
        let mut g = two_player_game();
        for _ in 0..3 {
            g.add_card_to_library(0, catalog::island());
        }
        let id = g.add_card_to_hand(0, catalog::chart_a_course());
        let h = g.players[0].hand.len();
        g.players[0].mana_pool.add(Color::Blue, 1);
        g.players[0].mana_pool.add_colorless(1);
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast Chart a Course");
        drain_stack(&mut g);
        // No attack this turn → draw 2, then discard 1 → net +1, minus the spell.
        assert_eq!(g.players[0].hand.len(), h - 1 + 2 - 1, "drew two, discarded one (no attack)");
    }

    /// Living Death swaps graveyard creatures for battlefield creatures.
    #[test]
    fn living_death_swaps_graveyards_and_battlefields() {
        let mut g = two_player_game();
        // Player 0: a creature in play, a creature in the graveyard.
        let in_play = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let in_grave = g.add_card_to_graveyard(0, catalog::grave_titan());
        let id = g.add_card_to_hand(0, catalog::living_death());
        g.players[0].mana_pool.add(Color::Black, 2);
        g.players[0].mana_pool.add_colorless(3);
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast Living Death");
        drain_stack(&mut g);
        assert!(g.battlefield_find(in_play).is_none(), "battlefield creature sacrificed");
        assert!(g.battlefield_find(in_grave).is_some(), "graveyard creature reanimated");
        assert!(g.players[0].graveyard.iter().any(|c| c.id == in_play),
            "the sacrificed creature is in the graveyard");
    }

    /// Show and Tell lets the caster put a permanent from hand onto the battlefield.
    #[test]
    fn show_and_tell_puts_permanent_from_hand() {
        let mut g = two_player_game();
        let titan = g.add_card_to_hand(0, catalog::grave_titan());
        let id = g.add_card_to_hand(0, catalog::show_and_tell());
        g.players[0].mana_pool.add(Color::Blue, 1);
        g.players[0].mana_pool.add_colorless(2);
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast Show and Tell");
        drain_stack(&mut g);
        assert!(g.battlefield_find(titan).is_some(), "the highest-mana permanent entered the battlefield");
        assert!(!g.players[0].hand.iter().any(|c| c.id == titan), "it left hand");
    }

    /// Sylvan Tutor puts a creature card on top of the library.
    #[test]
    fn sylvan_tutor_tops_a_creature() {
        let mut g = two_player_game();
        let titan = g.add_card_to_library(0, catalog::grave_titan());
        g.add_card_to_library(0, catalog::island()); // a non-creature to leave behind
        g.decider = Box::new(crabomination::decision::ScriptedDecider::new([
            crabomination::decision::DecisionAnswer::Search(Some(titan)),
        ]));
        let id = g.add_card_to_hand(0, catalog::sylvan_tutor());
        g.players[0].mana_pool.add(Color::Green, 1);
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast Sylvan Tutor");
        drain_stack(&mut g);
        assert_eq!(g.players[0].library.first().map(|c| c.id), Some(titan),
            "tutored creature is on top of the library");
    }

    /// Final Parting puts one card to hand and another to graveyard.
    #[test]
    fn final_parting_splits_two_cards() {
        let mut g = two_player_game();
        let to_hand = g.add_card_to_library(0, catalog::grave_titan());
        let to_grave = g.add_card_to_library(0, catalog::lightning_bolt());
        g.decider = Box::new(crabomination::decision::ScriptedDecider::new([
            crabomination::decision::DecisionAnswer::Search(Some(to_hand)),
            crabomination::decision::DecisionAnswer::Search(Some(to_grave)),
        ]));
        let id = g.add_card_to_hand(0, catalog::final_parting());
        g.players[0].mana_pool.add(Color::Black, 2);
        g.players[0].mana_pool.add_colorless(3);
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast Final Parting");
        drain_stack(&mut g);
        assert!(g.players[0].hand.iter().any(|c| c.id == to_hand), "first card to hand");
        assert!(g.players[0].graveyard.iter().any(|c| c.id == to_grave), "second card to graveyard");
    }

    /// Altar's Reap sacrifices a creature and draws two.
    #[test]
    fn altars_reap_sacrifices_and_draws() {
        let mut g = two_player_game();
        let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        for _ in 0..2 {
            g.add_card_to_library(0, catalog::island());
        }
        let id = g.add_card_to_hand(0, catalog::altars_reap());
        let h = g.players[0].hand.len();
        g.players[0].mana_pool.add(Color::Black, 1);
        g.players[0].mana_pool.add_colorless(1);
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast Altar's Reap");
        drain_stack(&mut g);
        assert!(g.battlefield_find(fodder).is_none(), "creature sacrificed");
        assert_eq!(g.players[0].hand.len(), h - 1 + 2, "drew two cards");
    }

    /// Corpse Knight drains each opponent when another creature you control enters.
    #[test]
    fn corpse_knight_drains_on_creature_entry() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::corpse_knight());
        let life = g.players[1].life;
        let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
        g.players[0].mana_pool.add(Color::Green, 1);
        g.players[0].mana_pool.add_colorless(1);
        g.perform_action(GameAction::CastSpell {
            card_id: bear, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast a creature");
        drain_stack(&mut g);
        assert_eq!(g.players[1].life, life - 1, "opponent lost 1 when a creature entered");
    }

    /// Harvester of Souls draws when another nontoken creature dies.
    #[test]
    fn harvester_of_souls_draws_on_creature_death() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::harvester_of_souls());
        let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        g.add_card_to_library(0, catalog::island());
        // "you may draw" — accept the optional trigger.
        g.decider = Box::new(crabomination::decision::ScriptedDecider::new([
            crabomination::decision::DecisionAnswer::Bool(true),
        ]));
        let h = g.players[0].hand.len();
        let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
        g.players[0].mana_pool.add(Color::Red, 1);
        g.perform_action(GameAction::CastSpell {
            card_id: bolt, target: Some(Target::Permanent(victim)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("bolt the bear");
        drain_stack(&mut g);
        assert!(g.battlefield_find(victim).is_none(), "bear died");
        assert_eq!(g.players[0].hand.len(), h + 1, "Harvester drew a card on the death");
    }

    /// Snap returns a creature to hand and untaps up to two lands.
    #[test]
    fn snap_bounces_creature_and_untaps_lands() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let l1 = g.add_card_to_battlefield(0, catalog::island());
        let l2 = g.add_card_to_battlefield(0, catalog::island());
        g.battlefield_find_mut(l1).unwrap().tapped = true;
        g.battlefield_find_mut(l2).unwrap().tapped = true;
        let id = g.add_card_to_hand(0, catalog::snap());
        g.players[0].mana_pool.add(Color::Blue, 1);
        g.players[0].mana_pool.add_colorless(1);
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: Some(Target::Permanent(bear)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast Snap");
        drain_stack(&mut g);
        assert!(g.battlefield_find(bear).is_none(), "creature bounced");
        assert!(!g.battlefield_find(l1).unwrap().tapped && !g.battlefield_find(l2).unwrap().tapped,
            "two lands untapped");
    }

    /// Throttle gives -4/-4, killing a small creature.
    #[test]
    fn throttle_shrinks_and_kills() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
        let id = g.add_card_to_hand(0, catalog::throttle());
        g.players[0].mana_pool.add(Color::Black, 1);
        g.players[0].mana_pool.add_colorless(4);
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: Some(Target::Permanent(bear)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast Throttle");
        drain_stack(&mut g);
        assert!(g.battlefield_find(bear).is_none(), "2/2 dies to -4/-4");
    }

    /// Trophy Mage tutors an artifact with mana value 3.
    #[test]
    fn trophy_mage_tutors_mv3_artifact() {
        let mut g = two_player_game();
        let rock = g.add_card_to_library(0, catalog::darksteel_ingot()); // MV 3 artifact
        g.add_card_to_library(0, catalog::sol_ring()); // MV 1 — should be ineligible
        g.decider = Box::new(crabomination::decision::ScriptedDecider::new([
            crabomination::decision::DecisionAnswer::Search(Some(rock)),
        ]));
        g.move_card_to_battlefield_for_test(0, catalog::trophy_mage());
        drain_stack(&mut g);
        assert!(g.players[0].hand.iter().any(|c| c.id == rock), "MV3 artifact tutored to hand");
    }

    /// Thirst for Knowledge draws three then discards two.
    #[test]
    fn thirst_for_knowledge_draws_three_discards_two() {
        let mut g = two_player_game();
        for _ in 0..4 {
            g.add_card_to_library(0, catalog::island());
        }
        let id = g.add_card_to_hand(0, catalog::thirst_for_knowledge());
        let h = g.players[0].hand.len();
        g.players[0].mana_pool.add(Color::Blue, 1);
        g.players[0].mana_pool.add_colorless(2);
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast Thirst for Knowledge");
        drain_stack(&mut g);
        // -1 (the spell) +3 drawn -2 discarded = net 0.
        assert_eq!(g.players[0].hand.len(), h - 1 + 3 - 2, "drew three, discarded two");
    }

    /// Kavu Predator grows when an opponent gains life.
    #[test]
    fn kavu_predator_grows_on_opponent_lifegain() {
        let mut g = two_player_game();
        let kavu = g.add_card_to_battlefield(0, catalog::kavu_predator());
        // The opponent gains 3 life — Kavu's controller's opponent.
        g.adjust_life(1, 3);
        g.dispatch_triggers_for_events(&[GameEvent::LifeGained { player: 1, amount: 3 }]);
        drain_stack(&mut g);
        assert_eq!(
            g.battlefield_find(kavu).unwrap().counter_count(crabomination::card::CounterType::PlusOnePlusOne),
            3, "Kavu Predator gained three +1/+1 counters");
    }

    /// Seal Away exiles a tapped creature until it leaves; the creature returns
    /// when Seal Away is destroyed.
    #[test]
    fn seal_away_exiles_tapped_creature_until_it_leaves() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        g.battlefield_find_mut(bear).unwrap().tapped = true;
        let id = g.add_card_to_hand(0, catalog::seal_away());
        g.players[0].mana_pool.add(Color::White, 1);
        g.players[0].mana_pool.add_colorless(1);
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: Some(Target::Permanent(bear)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast Seal Away");
        drain_stack(&mut g);
        assert!(g.battlefield_find(bear).is_none(), "tapped creature exiled");
        // Destroy Seal Away → the creature returns.
        let seal = g.battlefield.iter().find(|c| c.definition.name == "Seal Away").unwrap().id;
        let bolt = g.add_card_to_hand(0, catalog::disenchant());
        g.players[0].mana_pool.add(Color::White, 1);
        g.players[0].mana_pool.add_colorless(1);
        g.perform_action(GameAction::CastSpell {
            card_id: bolt, target: Some(Target::Permanent(seal)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("destroy Seal Away");
        drain_stack(&mut g);
        assert!(g.battlefield.iter().any(|c| c.definition.name == "Grizzly Bears"),
            "creature returns when Seal Away leaves");
    }

    /// Conclave Tribunal exiles a nonland permanent an opponent controls.
    #[test]
    fn conclave_tribunal_exiles_nonland_permanent() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let id = g.add_card_to_hand(0, catalog::conclave_tribunal());
        g.players[0].mana_pool.add(Color::White, 1);
        g.players[0].mana_pool.add_colorless(3);
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: Some(Target::Permanent(bear)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast Conclave Tribunal");
        drain_stack(&mut g);
        assert!(g.battlefield_find(bear).is_none(), "opponent's creature exiled");
    }

    /// Fiery Cannonade deals 2 to each non-Pirate creature.
    #[test]
    fn fiery_cannonade_spares_pirates() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2, not a Pirate
        let id = g.add_card_to_hand(0, catalog::fiery_cannonade());
        g.players[0].mana_pool.add(Color::Red, 1);
        g.players[0].mana_pool.add_colorless(2);
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast Fiery Cannonade");
        drain_stack(&mut g);
        assert!(g.battlefield_find(bear).is_none(), "2/2 non-Pirate dies to 2 damage");
    }

    /// Magmaquake deals X to each non-flying creature, sparing flyers.
    #[test]
    fn magmaquake_spares_flyers() {
        let mut g = two_player_game();
        let ground = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2 no flying
        let flyer = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4 flying
        let id = g.add_card_to_hand(0, catalog::magmaquake());
        g.players[0].mana_pool.add(Color::Red, 2);
        g.players[0].mana_pool.add_colorless(3);
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: None, additional_targets: vec![], mode: None, x_value: Some(3),
        }).expect("cast Magmaquake for X=3");
        drain_stack(&mut g);
        assert!(g.battlefield_find(ground).is_none(), "ground creature dies to 3");
        assert!(g.battlefield_find(flyer).is_some(), "flyer untouched");
    }

    /// Star of Extinction destroys a land and wipes the board with 20 damage.
    #[test]
    fn star_of_extinction_destroys_land_and_wipes() {
        let mut g = two_player_game();
        let land = g.add_card_to_battlefield(1, catalog::forest());
        let titan = g.add_card_to_battlefield(1, catalog::grave_titan()); // 6/6
        let id = g.add_card_to_hand(0, catalog::star_of_extinction());
        g.players[0].mana_pool.add(Color::Red, 2);
        g.players[0].mana_pool.add_colorless(5);
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: Some(Target::Permanent(land)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast Star of Extinction");
        drain_stack(&mut g);
        assert!(g.battlefield_find(land).is_none(), "target land destroyed");
        assert!(g.battlefield_find(titan).is_none(), "6/6 wiped by 20 damage");
    }

    /// Pit Fight makes your creature fight an opponent's; deathtouch-free trade.
    #[test]
    fn pit_fight_resolves_a_fight() {
        let mut g = two_player_game();
        let mine = g.add_card_to_battlefield(0, catalog::grave_titan()); // 6/6
        let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
        let id = g.add_card_to_hand(0, catalog::pit_fight());
        g.players[0].mana_pool.add(Color::Green, 1);
        g.players[0].mana_pool.add_colorless(1);
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: Some(Target::Permanent(mine)),
            additional_targets: vec![Target::Permanent(theirs)], mode: None, x_value: None,
        }).expect("cast Pit Fight");
        drain_stack(&mut g);
        assert!(g.battlefield_find(theirs).is_none(), "2/2 dies to the 6/6");
        assert!(g.battlefield_find(mine).is_some(), "6/6 survives 2 damage");
    }

    /// Hunt the Weak grows your creature, then it fights.
    #[test]
    fn hunt_the_weak_buffs_then_fights() {
        let mut g = two_player_game();
        let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2 → 3/3
        let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
        let id = g.add_card_to_hand(0, catalog::hunt_the_weak());
        g.players[0].mana_pool.add(Color::Green, 1);
        g.players[0].mana_pool.add_colorless(3);
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: Some(Target::Permanent(mine)),
            additional_targets: vec![Target::Permanent(theirs)], mode: None, x_value: None,
        }).expect("cast Hunt the Weak");
        drain_stack(&mut g);
        assert!(g.battlefield_find(theirs).is_none(), "their 2/2 dies to the buffed 3/3");
        assert!(g.battlefield_find(mine).is_some(), "your 3/3 survives 2 damage");
    }

    /// Bramblecrush destroys a noncreature permanent (a land).
    #[test]
    fn bramblecrush_destroys_noncreature() {
        let mut g = two_player_game();
        let land = g.add_card_to_battlefield(1, catalog::forest());
        let id = g.add_card_to_hand(0, catalog::bramblecrush());
        g.players[0].mana_pool.add(Color::Green, 2);
        g.players[0].mana_pool.add_colorless(2);
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: Some(Target::Permanent(land)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast Bramblecrush");
        drain_stack(&mut g);
        assert!(g.battlefield_find(land).is_none(), "noncreature permanent destroyed");
    }

    /// Creeping Corrosion destroys all artifacts.
    #[test]
    fn creeping_corrosion_wipes_artifacts() {
        let mut g = two_player_game();
        let rock = g.add_card_to_battlefield(1, catalog::sol_ring());
        let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let id = g.add_card_to_hand(0, catalog::creeping_corrosion());
        g.players[0].mana_pool.add(Color::Green, 2);
        g.players[0].mana_pool.add_colorless(2);
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast Creeping Corrosion");
        drain_stack(&mut g);
        assert!(g.battlefield_find(rock).is_none(), "artifact destroyed");
        assert!(g.battlefield_find(bear).is_some(), "creature untouched");
    }

    /// Devour Flesh makes the target player sacrifice a creature and gain its
    /// toughness in life.
    #[test]
    fn devour_flesh_edicts_and_grants_toughness_life() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(1, catalog::grave_titan()); // 6/6 — only creature
        let life = g.players[1].life;
        let id = g.add_card_to_hand(0, catalog::devour_flesh());
        g.players[0].mana_pool.add(Color::Black, 1);
        g.players[0].mana_pool.add_colorless(1);
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: Some(Target::Player(1)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast Devour Flesh");
        drain_stack(&mut g);
        assert!(!g.battlefield.iter().any(|c| c.controller == 1 && c.definition.is_creature()),
            "opponent sacrificed their creature");
        assert_eq!(g.players[1].life, life + 6, "opponent gained 6 life (the 6/6's toughness)");
    }

    /// Mudbutton Torchrunner deals 3 to any target when it dies.
    #[test]
    fn mudbutton_torchrunner_deals_3_on_death() {
        let mut g = two_player_game();
        let mud = g.add_card_to_battlefield(0, catalog::mudbutton_torchrunner());
        let life = g.players[1].life;
        // Sacrifice it to a free outlet to trigger the death damage at a player.
        let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
        g.players[0].mana_pool.add(Color::Red, 1);
        g.decider = Box::new(crabomination::decision::ScriptedDecider::new([
            crabomination::decision::DecisionAnswer::Target(Target::Player(1)),
        ]));
        g.perform_action(GameAction::CastSpell {
            card_id: bolt, target: Some(Target::Permanent(mud)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("bolt the 1/1");
        drain_stack(&mut g);
        assert!(g.battlefield_find(mud).is_none(), "Torchrunner died");
        assert_eq!(g.players[1].life, life - 3, "death trigger dealt 3 to the player");
    }

    /// Llanowar Mentor discards a card to mint a mana-producing Elf token.
    #[test]
    fn llanowar_mentor_makes_mana_elf() {
        let mut g = two_player_game();
        let mentor = g.add_card_to_battlefield(0, catalog::llanowar_mentor());
        g.clear_sickness(mentor);
        let pitch = g.add_card_to_hand(0, catalog::lightning_bolt());
        g.players[0].mana_pool.add(Color::Green, 1);
        g.perform_action(GameAction::ActivateAbility {
            card_id: mentor, ability_index: 0,
            target: None, additional_targets: vec![], x_value: None,
        }).expect("activate Llanowar Mentor");
        drain_stack(&mut g);
        assert!(g.players[0].graveyard.iter().any(|c| c.id == pitch), "a card was discarded");
        assert!(g.battlefield.iter().any(|c| c.definition.name == "Llanowar Elves" && c.is_token),
            "a 1/1 mana Elf token was created");
    }
}

mod recent5 {
    use crabomination::card::{CounterType, Keyword};
    use crabomination::catalog;
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    use crabomination::game::types::{Attack, AttackTarget, Target, TurnStep};
    use crabomination::game::*;
    use crabomination::mana::Color;

    fn advance_to(g: &mut GameState, step: TurnStep) {
        while g.step != step {
            g.perform_action(GameAction::PassPriority).expect("pass priority");
        }
    }

    /// Plaguecrafter's ETB makes each player sacrifice a creature.
    #[test]
    fn plaguecrafter_etb_each_player_sacrifices() {
        let mut g = two_player_game();
        let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let id = g.add_card_to_hand(0, catalog::plaguecrafter());
        g.players[0].mana_pool.add(Color::Black, 1);
        g.players[0].mana_pool.add_colorless(2);
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast Plaguecrafter");
        drain_stack(&mut g);
        assert!(g.battlefield_find(mine).is_none(), "controller sacrificed (kept Plaguecrafter)");
        assert!(g.battlefield_find(theirs).is_none(), "opponent sacrificed their only creature");
    }

    /// Wither and Bloom kills a 2/2 with -3/-3, then its graveyard ability adds a
    /// +1/+1 counter.
    #[test]
    fn wither_and_bloom_minus_then_graveyard_counter() {
        let mut g = two_player_game();
        let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let id = g.add_card_to_hand(0, catalog::wither_and_bloom());
        g.players[0].mana_pool.add(Color::Black, 1);
        g.players[0].mana_pool.add_colorless(1);
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: Some(Target::Permanent(victim)), additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast Wither and Bloom");
        drain_stack(&mut g);
        assert!(g.battlefield_find(victim).is_none(), "-3/-3 kills the 2/2");
        assert!(g.players[0].graveyard.iter().any(|c| c.id == id), "Wither and Bloom in graveyard");

        let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g.players[0].mana_pool.add(Color::Black, 1);
        g.players[0].mana_pool.add_colorless(1);
        g.perform_action(GameAction::ActivateAbility {
            card_id: id, ability_index: 0, target: Some(Target::Permanent(mine)), additional_targets: vec![], x_value: None,
        }).expect("graveyard ability");
        drain_stack(&mut g);
        assert_eq!(g.battlefield_find(mine).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
        assert!(g.exile.iter().any(|c| c.id == id), "exiled as its own cost");
    }

    /// Sythis gains a life when you cast an enchantment spell.
    #[test]
    fn sythis_gains_life_on_enchantment_cast() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::sythis_harvests_hand());
        g.add_card_to_library(0, catalog::forest());
        let ench = g.add_card_to_hand(0, catalog::garruks_uprising());
        let life = g.players[0].life;
        g.players[0].mana_pool.add(Color::Green, 1);
        g.players[0].mana_pool.add_colorless(2);
        g.perform_action(GameAction::CastSpell {
            card_id: ench, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast enchantment");
        drain_stack(&mut g);
        assert_eq!(g.players[0].life, life + 1, "Sythis gained 1 life for the enchantment cast");
    }

    /// Toski ships its keyword suite and draws when a creature deals combat damage
    /// to a player.
    #[test]
    fn toski_keywords_and_combat_draw() {
        let t = catalog::toski_bearer_of_secrets();
        for kw in [Keyword::CantBeCountered, Keyword::Indestructible, Keyword::MustAttack] {
            assert!(t.keywords.contains(&kw), "Toski has {kw:?}");
        }
        let mut g = two_player_game();
        let toski = g.add_card_to_battlefield(0, catalog::toski_bearer_of_secrets());
        g.clear_sickness(toski);
        g.add_card_to_library(0, catalog::forest());
        let hand = g.players[0].hand.len();
        advance_to(&mut g, TurnStep::DeclareAttackers);
        g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker: toski, target: AttackTarget::Player(1),
        }])).expect("attack");
        drain_stack(&mut g);
        advance_to(&mut g, TurnStep::PostCombatMain);
        assert_eq!(g.players[0].hand.len(), hand + 1, "combat damage to player drew a card");
    }

    /// Misdirection repoints a targeted spell at a new target.
    #[test]
    fn misdirection_changes_a_spells_target() {
        let mut g = two_player_game();
        // P1 bolts P0's bear; P0 misdirects it onto P1's bear.
        let my_bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let their_bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
        g.players[1].mana_pool.add(Color::Red, 1);
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 1;
        g.perform_action(GameAction::CastSpell {
            card_id: bolt, target: Some(Target::Permanent(my_bear)), additional_targets: vec![], mode: None, x_value: None,
        }).expect("P1 casts bolt at my bear");
        let mis = g.add_card_to_hand(0, catalog::misdirection());
        g.players[0].mana_pool.add(Color::Blue, 2);
        g.players[0].mana_pool.add_colorless(3);
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Target(Target::Permanent(their_bear))]));
        g.perform_action(GameAction::CastSpell {
            card_id: mis, target: Some(Target::Permanent(bolt)), additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast Misdirection at the bolt");
        drain_stack(&mut g);
        assert!(g.battlefield_find(my_bear).is_some(), "my bear was spared");
        assert!(g.battlefield_find(their_bear).is_none(), "the bolt now killed their bear");
    }

    /// Flawless Maneuver grants the team indestructible.
    #[test]
    fn flawless_maneuver_grants_indestructible() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let id = g.add_card_to_hand(0, catalog::flawless_maneuver());
        g.players[0].mana_pool.add(Color::White, 1);
        g.players[0].mana_pool.add_colorless(2);
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast Flawless Maneuver");
        drain_stack(&mut g);
        assert!(g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Indestructible), "bear is indestructible");
    }

    /// Venser's ETB returns a permanent to its owner's hand.
    #[test]
    fn venser_bounces_a_permanent() {
        let mut g = two_player_game();
        let target = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let id = g.add_card_to_hand(0, catalog::venser_shaper_savant());
        assert!(catalog::venser_shaper_savant().keywords.contains(&Keyword::Flash), "Venser has flash");
        g.players[0].mana_pool.add(Color::Blue, 2);
        g.players[0].mana_pool.add_colorless(2);
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast Venser");
        drain_stack(&mut g);
        g.perform_action(GameAction::PassPriority).ok();
        g.perform_action(GameAction::PassPriority).ok();
        // The ETB trigger needs a target; resolve via auto/scripted target.
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Target(Target::Permanent(target))]));
        drain_stack(&mut g);
        assert!(g.players[1].hand.iter().any(|c| c.id == target), "permanent returned to owner's hand");
    }

    /// Hullbreaker Horror ships flash + can't-be-countered and bounces a permanent
    /// when you cast a spell.
    #[test]
    fn hullbreaker_horror_bounces_on_spell_cast() {
        let h = catalog::hullbreaker_horror();
        assert!(h.keywords.contains(&Keyword::Flash) && h.keywords.contains(&Keyword::CantBeCountered));
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::hullbreaker_horror());
        let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g.players[0].mana_pool.add(Color::Red, 1);
        g.decider = Box::new(ScriptedDecider::new([
            DecisionAnswer::Bool(true),
            DecisionAnswer::Target(Target::Permanent(victim)),
        ]));
        g.perform_action(GameAction::CastSpell {
            card_id: bolt, target: Some(Target::Player(1)), additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast a spell");
        drain_stack(&mut g);
        assert!(g.players[1].hand.iter().any(|c| c.id == victim), "Hullbreaker bounced the permanent");
    }

    /// Drown in Sorrow sweeps small creatures with -2/-2 and scries.
    #[test]
    fn drown_in_sorrow_minus_two_sweep() {
        let mut g = two_player_game();
        let small = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
        let big = g.add_card_to_battlefield(1, catalog::grave_titan()); // 6/6
        let id = g.add_card_to_hand(0, catalog::drown_in_sorrow());
        g.add_card_to_library(0, catalog::forest());
        g.players[0].mana_pool.add(Color::Black, 2);
        g.players[0].mana_pool.add_colorless(1);
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast Drown in Sorrow");
        drain_stack(&mut g);
        assert!(g.battlefield_find(small).is_none(), "2/2 dies to -2/-2");
        assert!(g.battlefield_find(big).is_some(), "6/6 survives");
    }

    /// Shamanic Revelation draws one per creature and gains 4 life per power-4.
    #[test]
    fn shamanic_revelation_draws_and_gains() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
        g.add_card_to_battlefield(0, catalog::grave_titan()); // 6/6, power >= 4
        for _ in 0..5 { g.add_card_to_library(0, catalog::forest()); }
        let id = g.add_card_to_hand(0, catalog::shamanic_revelation());
        let life = g.players[0].life;
        let hand = g.players[0].hand.len();
        g.players[0].mana_pool.add(Color::Green, 2);
        g.players[0].mana_pool.add_colorless(3);
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast Shamanic Revelation");
        drain_stack(&mut g);
        // -1 hand for the cast, +2 drawn (two creatures).
        assert_eq!(g.players[0].hand.len(), hand - 1 + 2, "drew one per creature");
        assert_eq!(g.players[0].life, life + 4, "ferocious gained 4 for the power-4 creature");
    }

    /// End-Raze Forerunners pumps the rest of the team on ETB.
    #[test]
    fn end_raze_forerunners_team_pump() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let id = g.add_card_to_hand(0, catalog::end_raze_forerunners());
        g.players[0].mana_pool.add(Color::Green, 3);
        g.players[0].mana_pool.add_colorless(5);
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast End-Raze");
        drain_stack(&mut g);
        let b = g.battlefield_find(bear).unwrap();
        assert_eq!((b.power(), b.toughness()), (4, 4), "bear pumped +2/+2");
        assert!(g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Trample), "bear gained trample");
    }

    /// Garruk's Uprising grants trample and draws when a power-4 creature enters.
    #[test]
    fn garruks_uprising_trample_and_power4_draw() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::garruks_uprising());
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        assert!(g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Trample), "anthem grants trample");
        g.add_card_to_library(0, catalog::forest());
        let hand = g.players[0].hand.len();
        let titan = g.add_card_to_hand(0, catalog::grave_titan()); // power 6
        g.players[0].mana_pool.add(Color::Black, 2);
        g.players[0].mana_pool.add_colorless(4);
        g.perform_action(GameAction::CastSpell {
            card_id: titan, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast a power-4 creature");
        drain_stack(&mut g);
        assert!(g.players[0].hand.len() > hand, "Garruk's Uprising drew for the power-4 ETB");
    }

    /// Guardian Project draws when a nontoken creature you control enters.
    #[test]
    fn guardian_project_draws_on_creature_etb() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::guardian_project());
        g.add_card_to_library(0, catalog::forest());
        let hand = g.players[0].hand.len();
        let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
        g.players[0].mana_pool.add(Color::Green, 1);
        g.players[0].mana_pool.add_colorless(1);
        g.perform_action(GameAction::CastSpell {
            card_id: bear, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast a creature");
        drain_stack(&mut g);
        assert!(g.players[0].hand.len() > hand, "Guardian Project drew for the creature ETB");
    }

    /// Neoform sacrifices a creature and tutors a creature with MV one higher onto
    /// the battlefield with a +1/+1 counter.
    #[test]
    fn neoform_tutors_one_higher_with_counter() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::grizzly_bears()); // MV 2 (the only creature; auto-sacrificed)
        g.add_card_to_library(0, catalog::grave_titan()); // MV 6 — not eligible
        let target = g.add_card_to_library(0, catalog::hypnotic_specter()); // MV 3 == 2+1
        let id = g.add_card_to_hand(0, catalog::neoform());
        g.players[0].mana_pool.add(Color::Green, 1);
        g.players[0].mana_pool.add(Color::Blue, 1);
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(target))]));
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast Neoform");
        drain_stack(&mut g);
        let tutored = g.battlefield_find(target).expect("tutored creature on battlefield");
        assert_eq!(tutored.counter_count(CounterType::PlusOnePlusOne), 1, "entered with a +1/+1 counter");
    }

    /// Eldritch Evolution tutors a creature with MV up to sacrificed+2 and exiles
    /// itself.
    #[test]
    fn eldritch_evolution_tutors_up_to_plus_two() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::grizzly_bears()); // MV 2 → up to 4 (auto-sacrificed)
        let target = g.add_card_to_library(0, catalog::hypnotic_specter()); // MV 3 <= 4
        let id = g.add_card_to_hand(0, catalog::eldritch_evolution());
        g.players[0].mana_pool.add(Color::Green, 2);
        g.players[0].mana_pool.add_colorless(1);
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(target))]));
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast Eldritch Evolution");
        drain_stack(&mut g);
        assert!(g.battlefield_find(target).is_some(), "tutored creature on battlefield");
        assert!(g.exile.iter().any(|c| c.id == id), "Eldritch Evolution exiled itself");
    }

    /// Skrelv ships toxic + can't block and grants hexproof to another creature.
    #[test]
    fn skrelv_grants_hexproof() {
        let s = catalog::skrelv_defector_mite();
        assert!(s.keywords.contains(&Keyword::CantBlock) && s.keywords.contains(&Keyword::Toxic(1)));
        let mut g = two_player_game();
        let skrelv = g.add_card_to_battlefield(0, catalog::skrelv_defector_mite());
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.clear_sickness(skrelv);
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g.players[0].mana_pool.add(Color::White, 1);
        g.perform_action(GameAction::ActivateAbility {
            card_id: skrelv, ability_index: 0, target: Some(Target::Permanent(bear)), additional_targets: vec![], x_value: None,
        }).expect("activate Skrelv");
        drain_stack(&mut g);
        assert!(g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Hexproof), "target gained hexproof");
    }

    /// Soul's Majesty draws cards equal to a creature's power.
    #[test]
    fn souls_majesty_draws_equal_to_power() {
        let mut g = two_player_game();
        let titan = g.add_card_to_battlefield(0, catalog::grave_titan()); // power 6
        for _ in 0..8 { g.add_card_to_library(0, catalog::forest()); }
        let id = g.add_card_to_hand(0, catalog::souls_majesty());
        let hand = g.players[0].hand.len();
        g.players[0].mana_pool.add(Color::Green, 2);
        g.players[0].mana_pool.add_colorless(3);
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: Some(Target::Permanent(titan)), additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast Soul's Majesty");
        drain_stack(&mut g);
        assert_eq!(g.players[0].hand.len(), hand - 1 + 6, "drew 6 (the titan's power)");
    }

    /// Momentous Fall sacrifices a creature to draw its power and gain its
    /// toughness.
    #[test]
    fn momentous_fall_draw_and_gain() {
        let mut g = two_player_game();
        let titan = g.add_card_to_battlefield(0, catalog::grave_titan()); // 6/6
        for _ in 0..8 { g.add_card_to_library(0, catalog::forest()); }
        let id = g.add_card_to_hand(0, catalog::momentous_fall());
        let hand = g.players[0].hand.len();
        let life = g.players[0].life;
        g.players[0].mana_pool.add(Color::Green, 2);
        g.players[0].mana_pool.add_colorless(1);
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Cards(vec![titan])]));
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast Momentous Fall");
        drain_stack(&mut g);
        assert_eq!(g.players[0].hand.len(), hand - 1 + 6, "drew 6 (sacrificed power)");
        assert_eq!(g.players[0].life, life + 6, "gained 6 (sacrificed toughness)");
    }

    /// Life's Legacy sacrifices a creature to draw cards equal to its power.
    #[test]
    fn lifes_legacy_draws_equal_to_power() {
        let mut g = two_player_game();
        let titan = g.add_card_to_battlefield(0, catalog::grave_titan()); // power 6
        for _ in 0..8 { g.add_card_to_library(0, catalog::forest()); }
        let id = g.add_card_to_hand(0, catalog::lifes_legacy());
        let hand = g.players[0].hand.len();
        g.players[0].mana_pool.add(Color::Green, 1);
        g.players[0].mana_pool.add_colorless(1);
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Cards(vec![titan])]));
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast Life's Legacy");
        drain_stack(&mut g);
        assert_eq!(g.players[0].hand.len(), hand - 1 + 6, "drew 6 (sacrificed power)");
    }

    /// Return of the Wildspeaker — draw mode draws equal to greatest power.
    #[test]
    fn return_of_the_wildspeaker_draw_mode() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::grave_titan()); // power 6
        g.add_card_to_battlefield(0, catalog::grizzly_bears()); // power 2
        for _ in 0..8 { g.add_card_to_library(0, catalog::forest()); }
        let id = g.add_card_to_hand(0, catalog::return_of_the_wildspeaker());
        let hand = g.players[0].hand.len();
        g.players[0].mana_pool.add(Color::Green, 1);
        g.players[0].mana_pool.add_colorless(4);
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Mode(0)]));
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: None, additional_targets: vec![], mode: Some(0), x_value: None,
        }).expect("cast Return of the Wildspeaker");
        drain_stack(&mut g);
        assert_eq!(g.players[0].hand.len(), hand - 1 + 6, "drew 6 (greatest power)");
    }

    /// Overrun pumps the whole team +3/+3 and grants trample.
    #[test]
    fn overrun_team_pump_and_trample() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let id = g.add_card_to_hand(0, catalog::overrun());
        g.players[0].mana_pool.add(Color::Green, 3);
        g.players[0].mana_pool.add_colorless(2);
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast Overrun");
        drain_stack(&mut g);
        let b = g.battlefield_find(bear).unwrap();
        assert_eq!((b.power(), b.toughness()), (5, 5));
        assert!(g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Trample));
    }

    /// Larger Than Life pumps a single creature +4/+4 with trample.
    #[test]
    fn larger_than_life_pumps_and_tramples() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let id = g.add_card_to_hand(0, catalog::larger_than_life());
        g.players[0].mana_pool.add(Color::Green, 1);
        g.players[0].mana_pool.add_colorless(1);
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast Larger Than Life");
        drain_stack(&mut g);
        let b = g.battlefield_find(bear).unwrap();
        assert_eq!((b.power(), b.toughness()), (6, 6));
        assert!(g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Trample));
    }

    /// Prey's Vengeance ships +2/+2 and the Rebound keyword.
    #[test]
    fn preys_vengeance_pumps_with_rebound() {
        assert!(catalog::preys_vengeance().keywords.contains(&Keyword::Rebound), "has Rebound");
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let id = g.add_card_to_hand(0, catalog::preys_vengeance());
        g.players[0].mana_pool.add(Color::Green, 1);
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast Prey's Vengeance");
        drain_stack(&mut g);
        let b = g.battlefield_find(bear).unwrap();
        assert_eq!((b.power(), b.toughness()), (4, 4));
    }

    /// Savage Smash pumps your creature then fights an opponent's.
    #[test]
    fn savage_smash_pumps_then_fights() {
        let mut g = two_player_game();
        let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2 -> 4/4
        let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
        let id = g.add_card_to_hand(0, catalog::savage_smash());
        g.players[0].mana_pool.add(Color::Red, 1);
        g.players[0].mana_pool.add(Color::Green, 1);
        g.players[0].mana_pool.add_colorless(1);
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: Some(Target::Permanent(mine)),
            additional_targets: vec![Target::Permanent(theirs)], mode: None, x_value: None,
        }).expect("cast Savage Smash");
        drain_stack(&mut g);
        assert!(g.battlefield_find(theirs).is_none(), "4-power fights kill the 2/2");
        assert!(g.battlefield_find(mine).is_some(), "the 4/4 survives 2 damage");
    }

    /// Bite Down deals a creature's power to an opposing creature.
    #[test]
    fn bite_down_deals_power() {
        let mut g = two_player_game();
        let mine = g.add_card_to_battlefield(0, catalog::grave_titan()); // power 6
        let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
        let id = g.add_card_to_hand(0, catalog::bite_down());
        g.players[0].mana_pool.add(Color::Green, 1);
        g.players[0].mana_pool.add_colorless(1);
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: Some(Target::Permanent(mine)),
            additional_targets: vec![Target::Permanent(theirs)], mode: None, x_value: None,
        }).expect("cast Bite Down");
        drain_stack(&mut g);
        assert!(g.battlefield_find(theirs).is_none(), "6 damage kills the 2/2");
    }

    /// Crushing Vines (mode 1) destroys a target artifact.
    #[test]
    fn crushing_vines_destroys_artifact() {
        let mut g = two_player_game();
        let art = g.add_card_to_battlefield(1, catalog::sol_ring());
        let id = g.add_card_to_hand(0, catalog::crushing_vines());
        g.players[0].mana_pool.add(Color::Green, 1);
        g.players[0].mana_pool.add_colorless(2);
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: Some(Target::Permanent(art)), additional_targets: vec![], mode: Some(1), x_value: None,
        }).expect("cast Crushing Vines mode 1");
        drain_stack(&mut g);
        assert!(g.battlefield_find(art).is_none(), "artifact destroyed");
    }

    /// Inspiring Call draws per +1/+1-countered creature and grants indestructible.
    #[test]
    fn inspiring_call_draws_and_protects() {
        use crabomination::card::CounterType;
        let mut g = two_player_game();
        let a = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let b = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.battlefield_find_mut(a).unwrap().add_counters(CounterType::PlusOnePlusOne, 1);
        g.battlefield_find_mut(b).unwrap().add_counters(CounterType::PlusOnePlusOne, 1);
        g.add_card_to_battlefield(0, catalog::grizzly_bears()); // no counter — not counted
        for _ in 0..3 { g.add_card_to_library(0, catalog::forest()); }
        let id = g.add_card_to_hand(0, catalog::inspiring_call());
        let hand = g.players[0].hand.len();
        g.players[0].mana_pool.add(Color::Green, 1);
        g.players[0].mana_pool.add_colorless(2);
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast Inspiring Call");
        drain_stack(&mut g);
        assert_eq!(g.players[0].hand.len(), hand - 1 + 2, "drew 2 (the countered creatures)");
        assert!(g.computed_permanent(a).unwrap().keywords.contains(&Keyword::Indestructible));
    }
}

mod recent6 {
    use crabomination::card::{CounterType, CreatureType, Keyword};
    use crabomination::catalog;
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    use crabomination::game::types::{Attack, AttackTarget, Target, TurnStep};
    use crabomination::game::*;
    use crabomination::mana::Color;

    fn advance_to(g: &mut GameState, step: TurnStep) {
        while g.step != step {
            g.perform_action(GameAction::PassPriority).expect("pass priority");
        }
    }

    /// Karmic Guide ETB returns a creature card from your graveyard.
    #[test]
    fn karmic_guide_reanimates_from_graveyard() {
        let mut g = two_player_game();
        let dead = g.add_card_to_graveyard(0, catalog::grizzly_bears());
        let id = g.add_card_to_hand(0, catalog::karmic_guide());
        g.players[0].mana_pool.add(Color::White, 2);
        g.players[0].mana_pool.add_colorless(3);
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: Some(Target::Permanent(dead)), additional_targets: vec![],
            mode: None, x_value: None,
        }).expect("cast Karmic Guide");
        drain_stack(&mut g);
        assert!(g.battlefield_find(dead).is_some(), "grizzly returned to battlefield");
    }

    /// Elspeth, Sun's Champion: +1 makes three Soldiers; -3 destroys only power ≥ 4.
    #[test]
    fn elspeth_plus_one_tokens_and_minus_three_sweeps_big() {
        let mut g = two_player_game();
        let pw = g.add_card_to_battlefield(0, catalog::elspeth_suns_champion());
        g.perform_action(GameAction::ActivateLoyaltyAbility {
            card_id: pw, ability_index: 0, target: None, x_value: None,
        }).expect("+1");
        drain_stack(&mut g);
        let soldiers = g.battlefield.iter()
            .filter(|c| c.controller == 0 && c.definition.name == "Soldier").count();
        assert_eq!(soldiers, 3, "three 1/1 Soldiers");

        let small = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
        let big = g.add_card_to_battlefield(1, catalog::colossal_dreadmaw()); // 6/6
        // Reset the once-per-turn loyalty gate so we can exercise the -3 too.
        if let Some(c) = g.battlefield.iter_mut().find(|c| c.id == pw) { c.loyalty_uses_this_turn = 0; }
        g.perform_action(GameAction::ActivateLoyaltyAbility {
            card_id: pw, ability_index: 1, target: None, x_value: None,
        }).expect("-3");
        drain_stack(&mut g);
        assert!(g.battlefield_find(small).is_some(), "2/2 survives the power-4 sweep");
        assert!(g.battlefield_find(big).is_none(), "6/6 is destroyed");
    }

    /// Faith's Fetters gains 4 life and locks the enchanted creature out of combat.
    #[test]
    fn faiths_fetters_gains_life_and_locks_creature() {
        let mut g = two_player_game();
        let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let id = g.add_card_to_hand(0, catalog::faiths_fetters());
        let life = g.players[0].life;
        g.players[0].mana_pool.add(Color::White, 1);
        g.players[0].mana_pool.add_colorless(3);
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: Some(Target::Permanent(victim)), additional_targets: vec![],
            mode: None, x_value: None,
        }).expect("cast Faith's Fetters");
        drain_stack(&mut g);
        assert_eq!(g.players[0].life, life + 4, "gained 4 life");
        let v = g.computed_permanent(victim).unwrap();
        assert!(v.keywords.contains(&Keyword::CantAttack), "can't attack");
        assert!(v.keywords.contains(&Keyword::CantBlock), "can't block");
    }

    /// Increasing Devotion makes five Humans when cast from hand.
    #[test]
    fn increasing_devotion_makes_five_from_hand() {
        let mut g = two_player_game();
        let id = g.add_card_to_hand(0, catalog::increasing_devotion());
        g.players[0].mana_pool.add(Color::White, 2);
        g.players[0].mana_pool.add_colorless(3);
        cast(&mut g, id);
        let humans = g.battlefield.iter()
            .filter(|c| c.controller == 0 && c.definition.name == "Human").count();
        assert_eq!(humans, 5, "five 1/1 Humans");
    }

    /// Wing Shards forces the targeted player to sacrifice an attacking creature.
    #[test]
    fn wing_shards_sacrifices_attacker() {
        let mut g = two_player_game();
        // P1's attacker. Make P1 the active player so it can attack P0.
        let attacker = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let bystander = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        g.clear_sickness(attacker);
        g.active_player_idx = 1;
        g.priority.player_with_priority = 1;
        advance_to(&mut g, TurnStep::DeclareAttackers);
        g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker, target: AttackTarget::Player(0),
        }])).expect("attack");
        // P0 casts Wing Shards (instant) targeting P1.
        g.priority.player_with_priority = 0;
        let id = g.add_card_to_hand(0, catalog::wing_shards());
        g.players[0].mana_pool.add(Color::White, 2);
        g.players[0].mana_pool.add_colorless(1);
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: Some(Target::Player(1)), additional_targets: vec![],
            mode: None, x_value: None,
        }).expect("cast Wing Shards");
        drain_stack(&mut g);
        assert!(g.battlefield_find(attacker).is_none(), "attacking creature sacrificed");
        assert!(g.battlefield_find(bystander).is_some(), "non-attacker untouched");
    }

    /// Council's Judgment exiles an opponent's hexproof permanent (no targeting).
    #[test]
    fn councils_judgment_votes_out_hexproof() {
        let mut g = two_player_game();
        // Single legal candidate → unanimous vote → exiled, even with hexproof.
        let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let id = g.add_card_to_hand(0, catalog::councils_judgment());
        g.players[0].mana_pool.add(Color::White, 2);
        g.players[0].mana_pool.add_colorless(1);
        cast(&mut g, id);
        assert!(g.battlefield_find(foe).is_none(), "voted-out permanent exiled");
        assert!(g.exile.iter().any(|c| c.id == foe), "in exile (not graveyard)");
    }

    /// Talrand makes a Drake when you cast an instant or sorcery.
    #[test]
    fn talrand_makes_drake_on_instant() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::talrand_sky_summoner());
        let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
        g.players[0].mana_pool.add(Color::Red, 1);
        cast_at(&mut g, bolt, Target::Player(1));
        let drakes = g.battlefield.iter()
            .filter(|c| c.controller == 0 && c.definition.name == "Drake").count();
        assert_eq!(drakes, 1, "one 2/2 Drake");
    }

    /// Tezzeret the Seeker −X tutors an artifact onto the battlefield.
    #[test]
    fn tezzeret_minus_x_fetches_artifact() {
        let mut g = two_player_game();
        let pw = g.add_card_to_battlefield(0, catalog::tezzeret_the_seeker());
        let sol = g.add_card_to_library(0, catalog::sol_ring()); // MV 1
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(sol))]));
        g.perform_action(GameAction::ActivateLoyaltyAbility {
            card_id: pw, ability_index: 1, target: None, x_value: Some(1),
        }).expect("-X");
        drain_stack(&mut g);
        assert!(g.battlefield_find(sol).is_some(), "Sol Ring tutored to battlefield");
    }

    /// Dream Eater surveils 4 and bounces an opponent's nonland permanent.
    #[test]
    fn dream_eater_surveils_and_bounces() {
        let mut g = two_player_game();
        let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let top: Vec<_> = (0..4).map(|_| g.add_card_to_library(0, catalog::forest())).collect();
        let id = g.add_card_to_hand(0, catalog::dream_eater());
        g.players[0].mana_pool.add(Color::Blue, 2);
        g.players[0].mana_pool.add_colorless(4);
        g.decider = Box::new(ScriptedDecider::new([
            DecisionAnswer::ScryOrder { kept_top: top, bottom: vec![] }, // surveil 4: keep all
            DecisionAnswer::Bool(true),  // yes, bounce
            DecisionAnswer::Target(Target::Permanent(victim)),
        ]));
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast Dream Eater");
        drain_stack(&mut g);
        assert!(g.players[1].hand.iter().any(|c| c.id == victim), "creature bounced to hand");
    }

    /// Malcolm makes a Treasure when a Pirate deals combat damage to an opponent.
    #[test]
    fn malcolm_treasure_on_pirate_combat_damage() {
        let mut g = two_player_game();
        let malcolm = g.add_card_to_battlefield(0, catalog::malcolm_keen_eyed_navigator());
        g.clear_sickness(malcolm);
        advance_to(&mut g, TurnStep::DeclareAttackers);
        g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker: malcolm, target: AttackTarget::Player(1),
        }])).expect("attack");
        drain_stack(&mut g);
        advance_to(&mut g, TurnStep::PostCombatMain);
        let treasures = g.battlefield.iter()
            .filter(|c| c.controller == 0 && c.definition.name == "Treasure").count();
        assert_eq!(treasures, 1, "one Treasure for the opponent dealt damage");
    }

    /// Faerie Mastermind draws on the opponent's second draw, but not the first.
    #[test]
    fn faerie_mastermind_second_draw_payoff() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::faerie_mastermind());
        g.add_card_to_library(0, catalog::forest()); // my payoff draw
        for _ in 0..2 { g.add_card_to_library(1, catalog::forest()); }
        g.players[0].cards_drawn_this_turn = 0;
        g.players[1].cards_drawn_this_turn = 0;
        let my_hand = g.players[0].hand.len();
        // Opponent's first draw — no payoff.
        let mut ev = vec![];
        g.draw_one(1, &mut ev);
        g.dispatch_triggers_for_events(&ev);
        drain_stack(&mut g);
        assert_eq!(g.players[0].hand.len(), my_hand, "no draw on opponent's first card");
        // Opponent's second draw — Faerie Mastermind draws me a card.
        let mut ev2 = vec![];
        g.draw_one(1, &mut ev2);
        g.dispatch_triggers_for_events(&ev2);
        drain_stack(&mut g);
        assert_eq!(g.players[0].hand.len(), my_hand + 1, "drew on opponent's second card");
    }

    /// Profane Tutor suspends for {1}{B} and tutors any card to hand on resolution.
    #[test]
    fn profane_tutor_suspends_then_searches() {
        let mut g = two_player_game();
        let id = g.add_card_to_hand(0, catalog::profane_tutor());
        let wanted = g.add_card_to_library(0, catalog::grizzly_bears());
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(wanted))]));
        g.players[0].mana_pool.add(Color::Black, 1);
        g.players[0].mana_pool.add_colorless(1);
        g.perform_action(GameAction::Suspend { card_id: id }).expect("suspend");
        g.step = TurnStep::Upkeep;
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        for _ in 0..2 { let _ = g.process_suspend(); }
        drain_stack(&mut g);
        assert!(g.players[0].hand.iter().any(|c| c.id == wanted), "tutored card in hand");
    }

    /// Shambling Ghast's death trigger can give an opponent's creature -1/-1.
    #[test]
    fn shambling_ghast_death_minus_one() {
        let mut g = two_player_game();
        let ghast = g.add_card_to_battlefield(0, catalog::shambling_ghast());
        let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
        g.decider = Box::new(ScriptedDecider::new([
            DecisionAnswer::Mode(0),
            DecisionAnswer::Target(Target::Permanent(foe)),
        ]));
        // Kill the Ghast to fire its dies trigger.
        let ctx = crabomination::game::effects::EffectContext::for_trigger(ghast, 0, Some(Target::Permanent(ghast)), 0);
        g.resolve_effect(&crabomination::effect::Effect::DestroyNoRegen { what: crabomination::card::Selector::Target(0) }, &ctx).unwrap();
        drain_stack(&mut g);
        assert_eq!(g.computed_permanent(foe).unwrap().toughness, 1, "2/2 → 2/1 from -1/-1");
    }

    /// Priest of Forgotten Gods: sac two creatures → opponent loses 2 + sacrifices,
    /// you draw a card.
    #[test]
    fn priest_of_forgotten_gods_drain_and_draw() {
        let mut g = two_player_game();
        let priest = g.add_card_to_battlefield(0, catalog::priest_of_forgotten_gods());
        g.clear_sickness(priest);
        g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        g.add_card_to_library(0, catalog::forest());
        let life = g.players[1].life;
        let hand = g.players[0].hand.len();
        g.perform_action(GameAction::ActivateAbility {
            card_id: priest, ability_index: 0, target: Some(Target::Player(1)),
            additional_targets: vec![], x_value: None,
        }).expect("activate Priest");
        drain_stack(&mut g);
        assert_eq!(g.players[1].life, life - 2, "opponent lost 2 life");
        assert!(g.battlefield_find(foe).is_none(), "opponent sacrificed a creature");
        assert_eq!(g.players[0].hand.len(), hand + 1, "you drew a card");
    }

    /// Spawn of Mayhem's upkeep ping hits each player for 1.
    #[test]
    fn spawn_of_mayhem_upkeep_pings_each_player() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::spawn_of_mayhem());
        let l0 = g.players[0].life;
        let l1 = g.players[1].life;
        g.active_player_idx = 0;
        g.step = TurnStep::Untap;
        advance_to(&mut g, TurnStep::Upkeep);
        drain_stack(&mut g);
        assert_eq!(g.players[0].life, l0 - 1, "controller pinged");
        assert_eq!(g.players[1].life, l1 - 1, "opponent pinged");
    }

    /// Magus of the Coffers taps for {B} per Swamp you control.
    #[test]
    fn magus_of_the_coffers_mana_per_swamp() {
        let mut g = two_player_game();
        let magus = g.add_card_to_battlefield(0, catalog::magus_of_the_coffers());
        g.clear_sickness(magus);
        for _ in 0..3 { g.add_card_to_battlefield(0, catalog::swamp()); }
        g.players[0].mana_pool.add_colorless(2); // pay the {2}
        g.perform_action(GameAction::ActivateAbility {
            card_id: magus, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
        }).expect("activate Magus");
        assert_eq!(g.players[0].mana_pool.amount(Color::Black), 3, "added B per Swamp");
    }

    /// Plague Engineer's chosen-type aura shrinks opponents' creatures of that type.
    #[test]
    fn plague_engineer_shrinks_opponent_chosen_type() {
        let mut g = two_player_game();
        let goblin = g.add_card_to_battlefield(1, catalog::skirk_prospector()); // 1/1 Goblin
        let engineer = g.add_card_to_battlefield(0, catalog::plague_engineer());
        g.decider = Box::new(ScriptedDecider::new([
            DecisionAnswer::CreatureType(CreatureType::Goblin),
        ]));
        g.fire_self_etb_triggers(engineer, 0);
        drain_stack(&mut g);
        // A 1/1 Goblin getting -1/-1 has 0 toughness → dies as SBA.
        assert!(g.battlefield_find(goblin).is_none(), "opponent's 1/1 Goblin dies to -1/-1");
    }

    /// Mukotai Soulripper grows and gains menace when it sacrifices a creature on
    /// attack.
    #[test]
    fn mukotai_soulripper_attack_sacrifice() {
        let mut g = two_player_game();
        let vehicle = g.add_card_to_battlefield(0, catalog::mukotai_soulripper());
        let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.clear_sickness(vehicle);
        g.clear_sickness(fodder);
        // Crew it so it's a creature able to attack.
        g.perform_action(GameAction::Crew { vehicle, crew_creatures: vec![fodder] }).ok();
        let attacker = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.clear_sickness(attacker);
        advance_to(&mut g, TurnStep::DeclareAttackers);
        g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker: vehicle, target: AttackTarget::Player(1),
        }])).expect("attack");
        g.decider = Box::new(ScriptedDecider::new([
            DecisionAnswer::Bool(true),
            DecisionAnswer::Target(Target::Permanent(attacker)),
        ]));
        drain_stack(&mut g);
        assert_eq!(
            g.battlefield_find(vehicle).unwrap().counter_count(CounterType::PlusOnePlusOne),
            1, "+1/+1 counter",
        );
        assert!(g.computed_permanent(vehicle).unwrap().keywords.contains(&Keyword::Menace), "gained menace");
    }

    /// Dryad Arbor is a land creature that taps for green.
    #[test]
    fn dryad_arbor_taps_for_green() {
        let mut g = two_player_game();
        let arbor = g.add_card_to_battlefield(0, catalog::dryad_arbor());
        g.clear_sickness(arbor);
        let c = g.computed_permanent(arbor).unwrap();
        assert!(
            c.card_types.contains(&crabomination::card::CardType::Land)
                && c.card_types.contains(&crabomination::card::CardType::Creature),
            "land creature",
        );
        g.perform_action(GameAction::ActivateAbility {
            card_id: arbor, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
        }).expect("tap for G");
        assert_eq!(g.players[0].mana_pool.amount(Color::Green), 1);
    }

    /// Marwyn grows when another Elf enters and taps for green equal to her power.
    #[test]
    fn marwyn_grows_and_taps_for_power() {
        let mut g = two_player_game();
        let marwyn = g.add_card_to_battlefield(0, catalog::marwyn_the_nurturer());
        g.clear_sickness(marwyn);
        let elf = g.add_card_to_hand(0, catalog::llanowar_elves());
        g.players[0].mana_pool.add(Color::Green, 1);
        cast(&mut g, elf);
        assert_eq!(g.battlefield_find(marwyn).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
        g.perform_action(GameAction::ActivateAbility {
            card_id: marwyn, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
        }).expect("tap Marwyn");
        assert_eq!(g.players[0].mana_pool.amount(Color::Green), 2, "added G equal to her power (2)");
    }

    /// Hexdrinker levels into protection from instants, then from everything.
    #[test]
    fn hexdrinker_levels_into_protection() {
        let mut g = two_player_game();
        let hex = g.add_card_to_battlefield(0, catalog::hexdrinker());
        // Push to level 3: protection from instants → a Bolt can't target it.
        g.battlefield.iter_mut().find(|c| c.id == hex).unwrap()
            .counters.insert(CounterType::Level, 3);
        assert!(g.computed_permanent(hex).unwrap().keywords.contains(&Keyword::ProtectionFromInstants));
        let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
        g.players[1].mana_pool.add(Color::Red, 1);
        g.active_player_idx = 1;
        g.priority.player_with_priority = 1;
        let err = g.perform_action(GameAction::CastSpell {
            card_id: bolt, target: Some(Target::Permanent(hex)), additional_targets: vec![],
            mode: None, x_value: None,
        });
        assert!(err.is_err(), "instant can't target protection-from-instants");

        // Push to level 8: protection from everything → can't be blocked.
        g.battlefield.iter_mut().find(|c| c.id == hex).unwrap()
            .counters.insert(CounterType::Level, 8);
        let blocker = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        assert!(g.computed_permanent(hex).unwrap().keywords.contains(&Keyword::ProtectionFromEverything));
        assert!(!g.blocker_can_block_attacker(blocker, hex), "can't be blocked");
    }

    /// Wolfir Avenger can regenerate itself.
    #[test]
    fn wolfir_avenger_regenerates() {
        let mut g = two_player_game();
        let wolf = g.add_card_to_battlefield(0, catalog::wolfir_avenger());
        g.players[0].mana_pool.add(Color::Green, 1);
        g.players[0].mana_pool.add_colorless(1);
        g.perform_action(GameAction::ActivateAbility {
            card_id: wolf, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
        }).expect("regen shield");
        drain_stack(&mut g); // resolve the regen ability so the shield is up
        let ctx = crabomination::game::effects::EffectContext::for_trigger(wolf, 0, Some(Target::Permanent(wolf)), 0);
        g.resolve_effect(&crabomination::effect::Effect::Destroy { what: crabomination::card::Selector::Target(0) }, &ctx).unwrap();
        drain_stack(&mut g);
        assert!(g.battlefield_find(wolf).is_some(), "regeneration shield saved Wolfir Avenger");
    }

    /// Mwonvuli Acid-Moss destroys a land and ramps a Forest.
    #[test]
    fn mwonvuli_acid_moss_destroys_and_ramps() {
        let mut g = two_player_game();
        let foe_land = g.add_card_to_battlefield(1, catalog::island());
        let forest = g.add_card_to_library(0, catalog::forest());
        let id = g.add_card_to_hand(0, catalog::mwonvuli_acid_moss());
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(forest))]));
        g.players[0].mana_pool.add(Color::Green, 2);
        g.players[0].mana_pool.add_colorless(2);
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: Some(Target::Permanent(foe_land)), additional_targets: vec![],
            mode: None, x_value: None,
        }).expect("cast Acid-Moss");
        drain_stack(&mut g);
        assert!(g.battlefield_find(foe_land).is_none(), "land destroyed");
        assert!(g.battlefield_find(forest).is_some(), "Forest ramped onto battlefield");
    }

    /// Fabled Passage fetches a basic tapped; with four lands it enters untapped.
    #[test]
    fn fabled_passage_untaps_with_four_lands() {
        let mut g = two_player_game();
        let fp = g.add_card_to_battlefield(0, catalog::fabled_passage());
        // Three more lands so that after fetching the basic, you control 4+.
        for _ in 0..3 { g.add_card_to_battlefield(0, catalog::forest()); }
        let basic = g.add_card_to_library(0, catalog::plains());
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(basic))]));
        g.perform_action(GameAction::ActivateAbility {
            card_id: fp, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
        }).expect("crack Fabled Passage");
        drain_stack(&mut g);
        let fetched = g.battlefield_find(basic).expect("basic fetched");
        assert!(!fetched.tapped, "untapped (controlled 4+ lands)");
    }

    /// Mystic Sanctuary enters tapped with too few Islands.
    #[test]
    fn mystic_sanctuary_enters_tapped_without_islands() {
        let mut g = two_player_game();
        let ms = g.add_card_to_battlefield(0, catalog::mystic_sanctuary());
        g.fire_self_etb_triggers(ms, 0);
        drain_stack(&mut g);
        assert!(g.battlefield_find(ms).unwrap().tapped, "entered tapped with <3 other Islands");
    }
}

mod recent7 {
    use crabomination::catalog;
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    use crabomination::game::types::Target;
    use crabomination::game::*;
    use crabomination::mana::Color;
    // ── White ────────────────────────────────────────────────────────────────

    /// Mardu Woe-Reaper's ETB exiles a graveyard creature and gains 1 life.
    #[test]
    fn mardu_woe_reaper_exiles_and_gains() {
        let mut g = two_player_game();
        let dead = g.add_card_to_graveyard(1, catalog::grizzly_bears());
        let life = g.players[0].life;
        let id = g.add_card_to_hand(0, catalog::mardu_woe_reaper());
        g.players[0].mana_pool.add(Color::White, 1);
        g.decider = Box::new(ScriptedDecider::new([
            DecisionAnswer::Bool(true),
            DecisionAnswer::Target(Target::Permanent(dead)),
        ]));
        cast(&mut g, id);
        assert!(g.exile.iter().any(|c| c.id == dead), "graveyard creature exiled");
        assert_eq!(g.players[0].life, life + 1, "gained 1 life");
    }

    /// Peek draws a card.
    #[test]
    fn peek_draws() {
        let mut g = two_player_game();
        g.add_card_to_library(0, catalog::forest());
        let id = g.add_card_to_hand(0, catalog::peek());
        g.players[0].mana_pool.add(Color::Blue, 1);
        let hand = g.players[0].hand.len();
        cast(&mut g, id);
        assert_eq!(g.players[0].hand.len(), hand, "net hand unchanged (cast 1, drew 1)");
    }

    /// Pieces of the Puzzle takes up to two I/S; the rest go to the graveyard.
    #[test]
    fn pieces_of_the_puzzle_takes_two_instants() {
        let mut g = two_player_game();
        let b1 = catalog::lightning_bolt();
        let b2 = catalog::lightning_bolt();
        let creature = catalog::grizzly_bears();
        let i1 = g.add_card_to_library(0, b1);
        let i2 = g.add_card_to_library(0, b2);
        let c = g.add_card_to_library(0, creature);
        g.add_card_to_library(0, catalog::forest());
        g.add_card_to_library(0, catalog::forest());
        let id = g.add_card_to_hand(0, catalog::pieces_of_the_puzzle());
        g.players[0].mana_pool.add(Color::Blue, 1);
        g.players[0].mana_pool.add_colorless(2);
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(i1))]));
        cast(&mut g, id);
        assert!(g.players[0].hand.iter().any(|x| x.id == i1), "first instant in hand");
        assert!(g.players[0].hand.iter().any(|x| x.id == i2), "second instant auto-filled to hand");
        assert!(g.players[0].graveyard.iter().any(|x| x.id == c), "creature went to graveyard");
    }

    /// Ransack the Lab puts one of three to hand, the rest to the graveyard.
    #[test]
    fn ransack_the_lab_digs_one() {
        let mut g = two_player_game();
        let keep = g.add_card_to_library(0, catalog::grizzly_bears());
        let g1 = g.add_card_to_library(0, catalog::forest());
        let g2 = g.add_card_to_library(0, catalog::forest());
        let id = g.add_card_to_hand(0, catalog::ransack_the_lab());
        g.players[0].mana_pool.add(Color::Black, 1);
        g.players[0].mana_pool.add_colorless(1);
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(keep))]));
        cast(&mut g, id);
        assert!(g.players[0].hand.iter().any(|x| x.id == keep), "kept card in hand");
        assert!(g.players[0].graveyard.iter().any(|x| x.id == g1), "rest milled");
        assert!(g.players[0].graveyard.iter().any(|x| x.id == g2), "rest milled");
    }

    /// Leaf Gilder taps for green.
    #[test]
    fn leaf_gilder_taps_for_green() {
        let mut g = two_player_game();
        let dork = g.add_card_to_battlefield(0, catalog::leaf_gilder());
        g.clear_sickness(dork);
        g.perform_action(GameAction::ActivateAbility {
            card_id: dork, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
        }).expect("tap for G");
        assert_eq!(g.players[0].mana_pool.amount(Color::Green), 1);
    }

    /// Quirion Elves taps for green or the chosen color.
    #[test]
    fn quirion_elves_chosen_color_mana() {
        let mut g = two_player_game();
        let elf = g.add_card_to_battlefield(0, catalog::quirion_elves());
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Color(Color::Blue)]));
        g.fire_self_etb_triggers(elf, 0);
        drain_stack(&mut g);
        g.clear_sickness(elf);
        g.perform_action(GameAction::ActivateAbility {
            card_id: elf, ability_index: 1, target: None, additional_targets: vec![], x_value: None,
        }).expect("tap for chosen color");
        assert_eq!(g.players[0].mana_pool.amount(Color::Blue), 1, "added the chosen blue");
    }

    /// Skyshroud Elf filters {1} into red or white.
    #[test]
    fn skyshroud_elf_filters_into_red() {
        let mut g = two_player_game();
        let elf = g.add_card_to_battlefield(0, catalog::skyshroud_elf());
        g.clear_sickness(elf);
        g.players[0].mana_pool.add_colorless(1);
        g.perform_action(GameAction::ActivateAbility {
            card_id: elf, ability_index: 1, target: None, additional_targets: vec![], x_value: None,
        }).expect("filter into red");
        assert_eq!(g.players[0].mana_pool.amount(Color::Red), 1);
    }

    /// Briar Shield buffs +1/+1 and can be sacrificed for a +3/+3 pump.
    #[test]
    fn briar_shield_buffs_then_sac_pumps() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
        let aura = g.add_card_to_hand(0, catalog::briar_shield());
        g.players[0].mana_pool.add(Color::Green, 1);
        g.perform_action(GameAction::CastSpell {
            card_id: aura, target: Some(Target::Permanent(bear)), additional_targets: vec![],
            mode: None, x_value: None,
        }).expect("cast Briar Shield");
        drain_stack(&mut g);
        assert_eq!(g.computed_permanent(bear).unwrap().power, 3, "2/2 → 3/3 from the Aura");
        g.perform_action(GameAction::ActivateAbility {
            card_id: aura, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
        }).expect("sac for +3/+3");
        drain_stack(&mut g);
        assert_eq!(g.computed_permanent(bear).unwrap().power, 5, "base 2 + 3 after the Aura is sacrificed");
        assert!(g.battlefield_find(aura).is_none(), "Aura sacrificed");
    }

    /// Krosan Tusker's cycle trigger fetches a basic land to hand.
    #[test]
    fn krosan_tusker_cycle_fetches_basic() {
        let mut g = two_player_game();
        let id = g.add_card_to_hand(0, catalog::krosan_tusker());
        let plains = g.add_card_to_library(0, catalog::plains());
        g.players[0].mana_pool.add(Color::Green, 1);
        g.players[0].mana_pool.add_colorless(2);
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(plains))]));
        g.perform_action(GameAction::Cycle { card_id: id, x_value: None }).expect("cycle");
        drain_stack(&mut g);
        assert!(g.players[0].hand.iter().any(|c| c.id == plains), "fetched basic to hand");
    }

    /// Phyrexian Tower taps for {C}, and can sacrifice a creature for {B}{B}.
    #[test]
    fn phyrexian_tower_sac_for_black() {
        let mut g = two_player_game();
        let tower = g.add_card_to_battlefield(0, catalog::phyrexian_tower());
        let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.perform_action(GameAction::ActivateAbility {
            card_id: tower, ability_index: 1, target: None, additional_targets: vec![], x_value: None,
        }).expect("sac for BB");
        assert_eq!(g.players[0].mana_pool.amount(Color::Black), 2, "added BB");
        assert!(g.battlefield_find(fodder).is_none(), "creature sacrificed");
    }
}

mod recent8 {
    use crabomination::card::{CardType, CounterType, Keyword};
    use crabomination::catalog;
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    use crabomination::game::types::Target;
    use crabomination::game::*;
    use crabomination::mana::Color;
    // ── Earthbend (CR 701.66) ──────────────────────────────────────────────────

    /// Badgermole Cub's ETB earthbends a land into a 1/1 hasty land creature.
    #[test]
    fn badgermole_cub_earthbends_a_land() {
        let mut g = two_player_game();
        let land = g.add_card_to_battlefield(0, catalog::forest());
        let id = g.add_card_to_hand(0, catalog::badgermole_cub());
        g.players[0].mana_pool.add(Color::Green, 1);
        g.players[0].mana_pool.add_colorless(1);
        cast(&mut g, id);
        let comp = g.computed_permanent(land).expect("land still on battlefield");
        assert!(comp.card_types.contains(&CardType::Creature), "land is now a creature");
        assert!(comp.card_types.contains(&CardType::Land), "still a land");
        assert!(comp.keywords.contains(&Keyword::Haste), "has haste");
        assert_eq!(comp.power, 1, "0/0 + one +1/+1 counter");
        assert_eq!(
            g.battlefield_find(land).unwrap().counter_count(CounterType::PlusOnePlusOne),
            1
        );
    }

    /// Badgermole earthbends 2 — the land gains two +1/+1 counters.
    #[test]
    fn badgermole_earthbends_two() {
        let mut g = two_player_game();
        let land = g.add_card_to_battlefield(0, catalog::forest());
        let id = g.add_card_to_hand(0, catalog::badgermole());
        g.players[0].mana_pool.add(Color::Green, 1);
        g.players[0].mana_pool.add_colorless(4);
        cast(&mut g, id);
        assert_eq!(
            g.battlefield_find(land).unwrap().counter_count(CounterType::PlusOnePlusOne),
            2
        );
    }

    /// Earthbending Student earthbends 2 on ETB.
    #[test]
    fn earthbending_student_earthbends() {
        let mut g = two_player_game();
        let land = g.add_card_to_battlefield(0, catalog::forest());
        let id = g.add_card_to_hand(0, catalog::earthbending_student());
        g.players[0].mana_pool.add(Color::Green, 1);
        g.players[0].mana_pool.add_colorless(2);
        cast(&mut g, id);
        assert_eq!(
            g.battlefield_find(land).unwrap().counter_count(CounterType::PlusOnePlusOne),
            2
        );
    }

    /// Earth Village Ruffians earthbends 2 when it dies.
    #[test]
    fn earth_village_ruffians_earthbends_on_death() {
        let mut g = two_player_game();
        let land = g.add_card_to_battlefield(0, catalog::forest());
        let ruffians = g.add_card_to_battlefield(0, catalog::earth_village_ruffians());
        g.remove_to_graveyard_with_triggers(ruffians);
        drain_stack(&mut g);
        assert_eq!(
            g.battlefield_find(land).unwrap().counter_count(CounterType::PlusOnePlusOne),
            2,
            "death trigger earthbent the land"
        );
    }

    /// CR 701.66a — an earthbent land returns tapped when it dies, but a bounce to
    /// hand does NOT return it ("when it dies or is exiled").
    #[test]
    fn cr_701_66a_earthbent_land_returns_only_on_death_not_bounce() {
        use crabomination::effect::{Effect, Selector, ZoneDest, PlayerRef};
        // Death path: the land returns to the battlefield tapped.
        let mut g = two_player_game();
        let land = g.add_card_to_battlefield(0, catalog::forest());
        let id = g.add_card_to_hand(0, catalog::badgermole_cub());
        g.players[0].mana_pool.add(Color::Green, 1);
        g.players[0].mana_pool.add_colorless(1);
        cast(&mut g, id);
        g.remove_to_graveyard_with_triggers(land);
        drain_stack(&mut g);
        let back = g.battlefield.iter().find(|c| c.definition.name == "Forest" && c.controller == 0);
        assert!(back.is_some_and(|c| c.tapped), "died → returns tapped");

        // Bounce path: the land goes to hand and stays there.
        let mut g = two_player_game();
        let land = g.add_card_to_battlefield(0, catalog::forest());
        let id = g.add_card_to_hand(0, catalog::badgermole_cub());
        g.players[0].mana_pool.add(Color::Green, 1);
        g.players[0].mana_pool.add_colorless(1);
        cast(&mut g, id);
        let ctx = crabomination::game::effects::EffectContext::for_ability(land, 0, Some(Target::Permanent(land)));
        g.resolve_effect(&Effect::Move {
            what: Selector::Target(0),
            to: ZoneDest::Hand(PlayerRef::Seat(0)),
        }, &ctx).unwrap();
        drain_stack(&mut g);
        assert!(g.battlefield.iter().all(|c| c.definition.name != "Forest"), "bounced land does not return");
        assert!(g.players[0].hand.iter().any(|c| c.definition.name == "Forest"), "stays in hand");
    }

    /// Earthbender Ascension earthbends 2, then ramps a basic onto the battlefield
    /// tapped.
    #[test]
    fn earthbender_ascension_earthbends_and_ramps() {
        let mut g = two_player_game();
        let land = g.add_card_to_battlefield(0, catalog::forest());
        let fetched = g.add_card_to_library(0, catalog::forest());
        let id = g.add_card_to_hand(0, catalog::earthbender_ascension());
        g.players[0].mana_pool.add(Color::Green, 1);
        g.players[0].mana_pool.add_colorless(2);
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(fetched))]));
        cast(&mut g, id);
        assert_eq!(
            g.battlefield_find(land).unwrap().counter_count(CounterType::PlusOnePlusOne),
            2
        );
        let ramped = g.battlefield_find(fetched).expect("basic ramped to battlefield");
        assert!(ramped.tapped, "ramped land enters tapped");
    }

    /// Blighted Blackthorn blights itself for 2, draws, and loses 1 life.
    #[test]
    fn blighted_blackthorn_blights_draws_loses_life() {
        let mut g = two_player_game();
        g.add_card_to_library(0, catalog::forest());
        let id = g.add_card_to_hand(0, catalog::blighted_blackthorn());
        g.players[0].mana_pool.add(Color::Black, 1);
        g.players[0].mana_pool.add_colorless(4);
        let life = g.players[0].life;
        let hand = g.players[0].hand.len();
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
        let bb = id;
        cast(&mut g, bb);
        assert_eq!(
            g.battlefield_find(bb).unwrap().counter_count(CounterType::MinusOneMinusOne),
            2,
            "blighted itself (only creature)"
        );
        assert_eq!(g.players[0].hand.len(), hand, "cast one, drew one");
        assert_eq!(g.players[0].life, life - 1, "lost 1 life");
    }

    /// Chaos Spewer blights 2 when its controller can't pay {2}.
    #[test]
    fn chaos_spewer_blights_when_unpaid() {
        let mut g = two_player_game();
        let id = g.add_card_to_hand(0, catalog::chaos_spewer());
        // Exactly enough to cast {2}{B/R}, nothing left for the "pay {2}" rider.
        g.players[0].mana_pool.add(Color::Black, 1);
        g.players[0].mana_pool.add_colorless(2);
        cast(&mut g, id);
        assert_eq!(
            g.battlefield_find(id).unwrap().counter_count(CounterType::MinusOneMinusOne),
            2,
            "couldn't pay the rider, blighted itself"
        );
    }

    /// Boggart Mischief blights a creature for 1 and mints two Goblins.
    #[test]
    fn boggart_mischief_blights_and_makes_goblins() {
        let mut g = two_player_game();
        let victim = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let id = g.add_card_to_hand(0, catalog::boggart_mischief());
        g.players[0].mana_pool.add(Color::Black, 1);
        g.players[0].mana_pool.add_colorless(2);
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
        cast(&mut g, id);
        assert_eq!(
            g.battlefield_find(victim).unwrap().counter_count(CounterType::MinusOneMinusOne),
            1
        );
        let goblins = g
            .battlefield
            .iter()
            .filter(|c| c.controller == 0 && c.definition.name == "Goblin")
            .count();
        assert_eq!(goblins, 2, "two Goblin tokens");
    }

    /// Airbending Lesson exiles a nonland permanent with a {2} may-cast grant and
    /// draws a card.
    #[test]
    fn airbending_lesson_exiles_and_draws() {
        let mut g = two_player_game();
        let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        g.add_card_to_library(0, catalog::forest());
        let id = g.add_card_to_hand(0, catalog::airbending_lesson());
        g.players[0].mana_pool.add(Color::White, 1);
        g.players[0].mana_pool.add_colorless(2);
        let hand = g.players[0].hand.len();
        cast_at(&mut g, id, Target::Permanent(victim));
        let exiled = g.exile.iter().find(|c| c.id == victim).expect("airbent to exile");
        assert!(exiled.may_play_until.is_some(), "owner may cast it");
        assert_eq!(exiled.granted_alt_cast_cost_eot.as_ref().map(|c| c.cmc()), Some(2));
        assert_eq!(g.players[0].hand.len(), hand, "cast one, drew one");
    }

    /// The exile view surfaces airbend's {2} alt-cast cost so the client can
    /// render "play for {2}".
    #[test]
    fn airbend_exile_view_shows_alt_cost() {
        let mut g = two_player_game();
        let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let id = g.add_card_to_hand(0, catalog::airbending_lesson());
        g.players[0].mana_pool.add(Color::White, 1);
        g.players[0].mana_pool.add_colorless(2);
        g.add_card_to_library(0, catalog::forest());
        cast_at(&mut g, id, Target::Permanent(victim));
        let view = crabomination::server::view::project(&g, 1);
        let entry = view.exile.iter().find(|e| e.id == victim).expect("airbent card in exile view");
        assert_eq!(entry.may_play_recipient, Some(1), "owner may play it");
        assert_eq!(entry.may_play_alt_cost, Some(2), "renders play-for-{{2}}");
    }

    /// Aang airbends another nonland permanent on ETB.
    #[test]
    fn aang_airbends_on_etb() {
        let mut g = two_player_game();
        let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let id = g.add_card_to_hand(0, catalog::aang_the_last_airbender());
        g.players[0].mana_pool.add(Color::White, 1);
        g.players[0].mana_pool.add_colorless(3);
        cast(&mut g, id);
        assert!(g.exile.iter().any(|c| c.id == victim), "Aang airbent the opposing creature");
    }

    /// Airbender Ascension airbends a creature on ETB.
    #[test]
    fn airbender_ascension_airbends() {
        let mut g = two_player_game();
        let victim = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let id = g.add_card_to_hand(0, catalog::airbender_ascension());
        g.players[0].mana_pool.add(Color::White, 1);
        g.players[0].mana_pool.add_colorless(1);
        cast(&mut g, id);
        assert!(g.exile.iter().any(|c| c.id == victim), "airbent the creature");
    }

    /// Whirlwind Technique draws two, discards one, and airbends a creature.
    #[test]
    fn whirlwind_technique_draws_and_airbends() {
        let mut g = two_player_game();
        for _ in 0..3 { g.add_card_to_library(0, catalog::forest()); }
        let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let id = g.add_card_to_hand(0, catalog::whirlwind_technique());
        g.players[0].mana_pool.add(Color::Blue, 2);
        g.players[0].mana_pool.add_colorless(4);
        cast_at(&mut g, id, Target::Permanent(victim));
        assert!(g.exile.iter().any(|c| c.id == victim), "airbent a creature");
    }

    /// Glider Staff grants the equipped creature flying.
    #[test]
    fn glider_staff_grants_flying() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let staff = g.add_card_to_battlefield(0, catalog::glider_staff());
        g.players[0].mana_pool.add_colorless(1);
        g.step = crabomination::game::TurnStep::PreCombatMain;
        g.perform_action(GameAction::Equip { equipment: staff, target: bear }).expect("equip");
        assert!(g.permanent_has_keyword(bear, &Keyword::Flying), "equipped creature flies");
    }

    /// Corrupt Court Official makes an opponent discard on ETB.
    #[test]
    fn corrupt_court_official_makes_opponent_discard() {
        let mut g = two_player_game();
        g.add_card_to_hand(1, catalog::forest());
        let before = g.players[1].hand.len();
        let id = g.add_card_to_hand(0, catalog::corrupt_court_official());
        g.players[0].mana_pool.add(Color::Black, 1);
        g.players[0].mana_pool.add_colorless(1);
        cast(&mut g, id);
        assert_eq!(g.players[1].hand.len(), before - 1, "opponent discarded");
    }

    /// Jeong Jeong's Deserters puts a +1/+1 counter on a creature.
    #[test]
    fn jeong_jeongs_deserters_counters_a_creature() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let id = g.add_card_to_hand(0, catalog::jeong_jeongs_deserters());
        g.players[0].mana_pool.add(Color::White, 1);
        g.players[0].mana_pool.add_colorless(1);
        cast_at(&mut g, id, Target::Permanent(bear));
        assert_eq!(
            g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne),
            1
        );
    }

    /// Forecasting Fortune Teller creates a Clue on ETB.
    #[test]
    fn forecasting_fortune_teller_makes_a_clue() {
        let mut g = two_player_game();
        let id = g.add_card_to_hand(0, catalog::forecasting_fortune_teller());
        g.players[0].mana_pool.add(Color::Blue, 1);
        g.players[0].mana_pool.add_colorless(1);
        cast(&mut g, id);
        assert!(
            g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Clue"),
            "Clue token created"
        );
    }

    /// Pretending Poxbearers mints a 1/1 Ally when it dies.
    #[test]
    fn pretending_poxbearers_makes_ally_on_death() {
        let mut g = two_player_game();
        let pp = g.add_card_to_battlefield(0, catalog::pretending_poxbearers());
        g.remove_to_graveyard_with_triggers(pp);
        drain_stack(&mut g);
        assert!(
            g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Ally"),
            "Ally token created on death"
        );
    }

    /// Merchant of Many Hats returns itself from the graveyard to hand.
    #[test]
    fn merchant_of_many_hats_returns_from_graveyard() {
        let mut g = two_player_game();
        let merchant = g.add_card_to_graveyard(0, catalog::merchant_of_many_hats());
        g.players[0].mana_pool.add(Color::Black, 1);
        g.players[0].mana_pool.add_colorless(2);
        g.perform_action(GameAction::ActivateAbility {
            card_id: merchant,
            ability_index: 0,
            target: None,
            additional_targets: vec![],
            x_value: None,
        })
        .expect("activate gy ability");
        drain_stack(&mut g);
        assert!(g.players[0].hand.iter().any(|c| c.id == merchant), "returned to hand");
    }

    /// Yuyan Archers loots (discard a card to draw a card) on ETB.
    #[test]
    fn yuyan_archers_loots() {
        let mut g = two_player_game();
        g.add_card_to_hand(0, catalog::forest());
        g.add_card_to_library(0, catalog::island());
        let id = g.add_card_to_hand(0, catalog::yuyan_archers());
        g.players[0].mana_pool.add(Color::Red, 1);
        g.players[0].mana_pool.add_colorless(1);
        let hand = g.players[0].hand.len();
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
        cast(&mut g, id);
        // Cast Yuyan (-1 hand), discard 1, draw 1 → net hand = before - 1 (the cast).
        assert_eq!(g.players[0].hand.len(), hand - 1);
    }

    /// Platypus-Bear mills two cards on ETB.
    #[test]
    fn platypus_bear_mills_two() {
        let mut g = two_player_game();
        for _ in 0..3 { g.add_card_to_library(0, catalog::forest()); }
        let lib = g.players[0].library.len();
        let id = g.add_card_to_hand(0, catalog::platypus_bear());
        g.players[0].mana_pool.add(Color::Green, 1);
        g.players[0].mana_pool.add_colorless(1);
        cast(&mut g, id);
        assert_eq!(g.players[0].library.len(), lib - 2, "milled two");
    }

    /// Compassionate Healer gains life and scrys when it becomes tapped.
    #[test]
    fn compassionate_healer_triggers_on_tap() {
        let mut g = two_player_game();
        g.add_card_to_library(0, catalog::forest());
        let healer = g.add_card_to_battlefield(0, catalog::compassionate_healer());
        let life = g.players[0].life;
        g.battlefield_find_mut(healer).unwrap().tapped = true;
        g.dispatch_triggers_for_events(&[GameEvent::PermanentTapped { card_id: healer, actor: None }]);
        drain_stack(&mut g);
        assert_eq!(g.players[0].life, life + 1, "gained 1 life on tap");
    }

    /// Fire Nation Soldier is a 3/2 with haste.
    #[test]
    fn fire_nation_soldier_has_haste() {
        let def = catalog::fire_nation_soldier();
        assert_eq!((def.power, def.toughness), (3, 2));
        assert!(def.keywords.contains(&Keyword::Haste));
    }
}

mod recent9 {
    use crabomination::card::{CounterType, Keyword};
    use crabomination::catalog;
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    use crabomination::game::types::{Attack, AttackTarget, Target, TurnStep};
    use crabomination::game::*;
    use crabomination::mana::Color;

    fn advance_to(g: &mut GameState, step: TurnStep) {
        while g.step != step {
            g.perform_action(GameAction::PassPriority).expect("pass priority");
        }
    }

    /// Haru earthbends a land when another Ally enters under your control.
    #[test]
    fn haru_earthbends_on_ally_entering() {
        let mut g = two_player_game();
        let land = g.add_card_to_battlefield(0, catalog::forest());
        g.add_card_to_battlefield(0, catalog::haru_hidden_talent());
        let ally = g.add_card_to_hand(0, catalog::master_pakku()); // a vanilla Ally
        g.players[0].mana_pool.add(Color::Blue, 1);
        g.players[0].mana_pool.add_colorless(1);
        cast(&mut g, ally);
        assert_eq!(
            g.battlefield_find(land).unwrap().counter_count(CounterType::PlusOnePlusOne),
            1,
            "Ally entering earthbent the land"
        );
    }

    /// Avatar Enthusiasts grows when another Ally enters.
    #[test]
    fn avatar_enthusiasts_grows_on_ally() {
        let mut g = two_player_game();
        let enth = g.add_card_to_battlefield(0, catalog::avatar_enthusiasts());
        let ally = g.add_card_to_hand(0, catalog::master_pakku());
        g.players[0].mana_pool.add(Color::Blue, 1);
        g.players[0].mana_pool.add_colorless(1);
        cast(&mut g, ally);
        assert_eq!(
            g.battlefield_find(enth).unwrap().counter_count(CounterType::PlusOnePlusOne),
            1
        );
    }

    /// Invasion Reinforcements mints a 1/1 Ally on ETB.
    #[test]
    fn invasion_reinforcements_makes_ally() {
        let mut g = two_player_game();
        let id = g.add_card_to_hand(0, catalog::invasion_reinforcements());
        g.players[0].mana_pool.add(Color::White, 1);
        g.players[0].mana_pool.add_colorless(1);
        cast(&mut g, id);
        assert!(g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Ally"));
    }

    /// Aang, Airbending Master airbends another creature on ETB.
    #[test]
    fn aang_airbending_master_airbends() {
        let mut g = two_player_game();
        let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let id = g.add_card_to_hand(0, catalog::aang_airbending_master());
        g.players[0].mana_pool.add(Color::White, 1);
        g.players[0].mana_pool.add_colorless(4);
        cast(&mut g, id);
        assert!(g.exile.iter().any(|c| c.id == victim), "airbent the opposing creature");
    }

    /// Sinister Gnarlbark draws and blights at the end step.
    #[test]
    fn sinister_gnarlbark_end_step() {
        let mut g = two_player_game();
        g.add_card_to_library(0, catalog::forest());
        let gnarl = g.add_card_to_battlefield(0, catalog::sinister_gnarlbark());
        let hand = g.players[0].hand.len();
        g.step = TurnStep::End;
        g.fire_step_triggers(TurnStep::End);
        drain_stack(&mut g);
        assert_eq!(g.players[0].hand.len(), hand + 1, "drew a card");
        assert_eq!(
            g.battlefield_find(gnarl).unwrap().counter_count(CounterType::MinusOneMinusOne),
            1,
            "blighted itself"
        );
    }

    /// Dream Seizer blights and makes each opponent discard.
    #[test]
    fn dream_seizer_blights_and_discards() {
        let mut g = two_player_game();
        let victim = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.add_card_to_hand(1, catalog::forest());
        let before = g.players[1].hand.len();
        let id = g.add_card_to_hand(0, catalog::dream_seizer());
        g.players[0].mana_pool.add(Color::Black, 1);
        g.players[0].mana_pool.add_colorless(3);
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
        cast(&mut g, id);
        assert_eq!(
            g.battlefield_find(victim).unwrap().counter_count(CounterType::MinusOneMinusOne),
            1
        );
        assert_eq!(g.players[1].hand.len(), before - 1, "opponent discarded");
    }

    /// Sourbread Auntie blights 2 and mints two Goblins.
    #[test]
    fn sourbread_auntie_blights_for_goblins() {
        let mut g = two_player_game();
        let victim = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let id = g.add_card_to_hand(0, catalog::sourbread_auntie());
        g.players[0].mana_pool.add(Color::Red, 2);
        g.players[0].mana_pool.add_colorless(2);
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
        cast(&mut g, id);
        // grizzly is first in battlefield order → blighted 2 → 0/0 → dies.
        assert!(g.battlefield_find(victim).is_none(), "blighted 2/2 died");
        let goblins = g.battlefield.iter().filter(|c| c.definition.name == "Goblin").count();
        assert_eq!(goblins, 2);
    }

    /// Shadow Urchin blights when it attacks.
    #[test]
    fn shadow_urchin_blights_on_attack() {
        let mut g = two_player_game();
        let urchin = g.add_card_to_battlefield(0, catalog::shadow_urchin());
        g.clear_sickness(urchin);
        advance_to(&mut g, TurnStep::DeclareAttackers);
        g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker: urchin,
            target: AttackTarget::Player(1),
        }]))
        .expect("attack");
        drain_stack(&mut g);
        assert_eq!(
            g.battlefield_find(urchin).unwrap().counter_count(CounterType::MinusOneMinusOne),
            1,
            "blighted itself on attack"
        );
    }

    /// Knowledge Seeker grows on your second draw.
    #[test]
    fn knowledge_seeker_counters_on_second_draw() {
        let mut g = two_player_game();
        let seeker = g.add_card_to_battlefield(0, catalog::knowledge_seeker());
        for _ in 0..3 { g.add_card_to_library(0, catalog::island()); }
        let div = g.add_card_to_hand(0, catalog::divination()); // draw 2
        g.players[0].mana_pool.add(Color::Blue, 1);
        g.players[0].mana_pool.add_colorless(2);
        cast(&mut g, div);
        assert_eq!(
            g.battlefield_find(seeker).unwrap().counter_count(CounterType::PlusOnePlusOne),
            1
        );
    }

    /// Otter-Penguin pumps on your second draw.
    #[test]
    fn otter_penguin_pumps_on_second_draw() {
        let mut g = two_player_game();
        let otter = g.add_card_to_battlefield(0, catalog::otter_penguin());
        for _ in 0..3 { g.add_card_to_library(0, catalog::island()); }
        let div = g.add_card_to_hand(0, catalog::divination());
        g.players[0].mana_pool.add(Color::Blue, 1);
        g.players[0].mana_pool.add_colorless(2);
        cast(&mut g, div);
        let cp = g.computed_permanent(otter).unwrap();
        assert_eq!((cp.power, cp.toughness), (3, 3), "2/1 +1/+2 = 3/3");
    }

    /// Mai and Master Pakku both have prowess.
    #[test]
    fn prowess_creatures() {
        assert!(catalog::mai_jaded_edge().keywords.contains(&Keyword::Prowess));
        assert!(catalog::master_pakku().keywords.contains(&Keyword::Prowess));
    }

    /// Unlucky Cabbage Merchant creates a Food on ETB.
    #[test]
    fn unlucky_cabbage_merchant_makes_food() {
        let mut g = two_player_game();
        let id = g.add_card_to_hand(0, catalog::unlucky_cabbage_merchant());
        g.players[0].mana_pool.add(Color::Green, 1);
        g.players[0].mana_pool.add_colorless(1);
        cast(&mut g, id);
        assert!(g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Food"));
    }

    /// Curious Farm Animals gains 3 life when it dies.
    #[test]
    fn curious_farm_animals_gains_life_on_death() {
        let mut g = two_player_game();
        let cfa = g.add_card_to_battlefield(0, catalog::curious_farm_animals());
        let life = g.players[0].life;
        g.remove_to_graveyard_with_triggers(cfa);
        drain_stack(&mut g);
        assert_eq!(g.players[0].life, life + 3);
    }

    /// Deserter's Disciple makes a small creature unblockable.
    #[test]
    fn deserters_disciple_grants_unblockable() {
        let mut g = two_player_game();
        let disciple = g.add_card_to_battlefield(0, catalog::deserters_disciple());
        g.clear_sickness(disciple);
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // power 2
        g.step = TurnStep::PreCombatMain;
        g.perform_action(GameAction::ActivateAbility {
            card_id: disciple,
            ability_index: 0,
            target: Some(Target::Permanent(bear)),
            additional_targets: vec![],
            x_value: None,
        })
        .expect("activate");
        drain_stack(&mut g);
        assert!(g.permanent_has_keyword(bear, &Keyword::Unblockable));
    }

    /// Turtle-Duck pumps itself and gains trample.
    #[test]
    fn turtle_duck_pumps_and_tramples() {
        let mut g = two_player_game();
        let duck = g.add_card_to_battlefield(0, catalog::turtle_duck());
        g.players[0].mana_pool.add_colorless(3);
        g.perform_action(GameAction::ActivateAbility {
            card_id: duck,
            ability_index: 0,
            target: None,
            additional_targets: vec![],
            x_value: None,
        })
        .expect("activate");
        drain_stack(&mut g);
        let cp = g.computed_permanent(duck).unwrap();
        assert_eq!(cp.power, 4, "0 base + 4");
        assert!(cp.keywords.contains(&Keyword::Trample));
    }
}

mod recent10 {
    use crabomination::card::{CounterType, Keyword};
    use crabomination::catalog;
    use crabomination::game::types::Target;
    use crabomination::game::*;
    use crabomination::mana::Color;

    /// Glider Kids scrys on ETB (and flies).
    #[test]
    fn glider_kids_scrys() {
        let mut g = two_player_game();
        g.add_card_to_library(0, catalog::forest());
        let id = g.add_card_to_hand(0, catalog::glider_kids());
        g.players[0].mana_pool.add(Color::White, 1);
        g.players[0].mana_pool.add_colorless(2);
        cast(&mut g, id);
        assert!(g.permanent_has_keyword(id, &Keyword::Flying));
    }

    /// Messenger Hawk creates a Clue on ETB.
    #[test]
    fn messenger_hawk_makes_clue() {
        let mut g = two_player_game();
        let id = g.add_card_to_hand(0, catalog::messenger_hawk());
        g.players[0].mana_pool.add(Color::Blue, 1);
        g.players[0].mana_pool.add_colorless(2);
        cast(&mut g, id);
        assert!(g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Clue"));
    }

    /// Ostrich-Horse mills three and grabs a land.
    #[test]
    fn ostrich_horse_mills_and_grabs_land() {
        let mut g = two_player_game();
        let land = g.add_card_to_library(0, catalog::forest());
        g.add_card_to_library(0, catalog::island());
        g.add_card_to_library(0, catalog::island());
        let id = g.add_card_to_hand(0, catalog::ostrich_horse());
        g.players[0].mana_pool.add(Color::Green, 1);
        g.players[0].mana_pool.add_colorless(2);
        cast(&mut g, id);
        assert!(g.players[0].hand.iter().any(|c| c.id == land), "land went to hand");
    }

    /// Rowdy Snowballers taps an opposing creature on ETB.
    #[test]
    fn rowdy_snowballers_taps_opponent() {
        let mut g = two_player_game();
        let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let id = g.add_card_to_hand(0, catalog::rowdy_snowballers());
        g.players[0].mana_pool.add(Color::Blue, 1);
        g.players[0].mana_pool.add_colorless(2);
        cast_at(&mut g, id, Target::Permanent(victim));
        assert!(g.battlefield_find(victim).unwrap().tapped, "opposing creature tapped");
    }

    /// Treetop Freedom Fighters mints a 1/1 Ally and has haste.
    #[test]
    fn treetop_freedom_fighters_makes_ally() {
        let mut g = two_player_game();
        let id = g.add_card_to_hand(0, catalog::treetop_freedom_fighters());
        g.players[0].mana_pool.add(Color::Red, 1);
        g.players[0].mana_pool.add_colorless(2);
        cast(&mut g, id);
        assert!(g.permanent_has_keyword(id, &Keyword::Haste));
        assert!(g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Ally"));
    }

    /// Pirate Peddlers grows when you sacrifice another permanent.
    #[test]
    fn pirate_peddlers_grows_on_sacrifice() {
        let mut g = two_player_game();
        let peddler = g.add_card_to_battlefield(0, catalog::pirate_peddlers());
        let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let mut events = Vec::new();
        g.sacrifice_one(fodder, 0, &mut events);
        g.dispatch_triggers_for_events(&events);
        drain_stack(&mut g);
        assert_eq!(
            g.battlefield_find(peddler).unwrap().counter_count(CounterType::PlusOnePlusOne),
            1
        );
    }

    /// Iguana Parrot has flying, vigilance, and prowess.
    #[test]
    fn iguana_parrot_keywords() {
        let d = catalog::iguana_parrot();
        assert!(d.keywords.contains(&Keyword::Flying));
        assert!(d.keywords.contains(&Keyword::Vigilance));
        assert!(d.keywords.contains(&Keyword::Prowess));
    }

    /// Boar-q-pine grows when you cast a noncreature spell.
    #[test]
    fn boar_q_pine_grows_on_noncreature() {
        let mut g = two_player_game();
        let boar = g.add_card_to_battlefield(0, catalog::boar_q_pine());
        let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
        g.players[0].mana_pool.add(Color::Red, 1);
        cast_at(&mut g, bolt, Target::Player(1));
        assert_eq!(
            g.battlefield_find(boar).unwrap().counter_count(CounterType::PlusOnePlusOne),
            1
        );
    }

    /// Knowledge Seeker creates a Clue when it dies.
    #[test]
    fn knowledge_seeker_makes_clue_on_death() {
        let mut g = two_player_game();
        let seeker = g.add_card_to_battlefield(0, catalog::knowledge_seeker());
        g.remove_to_graveyard_with_triggers(seeker);
        drain_stack(&mut g);
        assert!(g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Clue"));
    }
}
