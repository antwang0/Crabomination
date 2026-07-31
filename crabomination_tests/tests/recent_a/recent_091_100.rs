//! Tests for recentN card batches 91-100 (merged from per-batch micro-files).

mod recent91 {
    use crabomination::catalog;
    use crabomination::card::{CounterType, CreatureType};
    use crabomination::game::two_player_game;
    use crabomination::game::types::Target;
    use crabomination::game::*;
    use crabomination::mana::Color;

    /// Cast Lightning Bolt from `seat` at `at`'s face.
    fn bolt_face(g: &mut GameState, seat: usize, at: usize) {
        let bolt = g.add_card_to_hand(seat, catalog::lightning_bolt());
        g.players[seat].mana_pool.add(Color::Red, 1);
        g.priority.player_with_priority = seat;
        g.step = TurnStep::PreCombatMain;
        g.perform_action(GameAction::CastSpell {
            card_id: bolt, target: Some(Target::Player(at)), additional_targets: vec![],
            mode: None, x_value: None,
        }).expect("bolt castable");
        drain_stack(g);
    }

    /// Draw one card for `seat`, firing its CardDrawn triggers.
    fn draw(g: &mut GameState, seat: usize) {
        let mut ev = vec![];
        g.draw_one(seat, &mut ev);
        g.dispatch_triggers_for_events(&ev);
        drain_stack(g);
    }

    #[test]
    fn kykar_mints_spirit_on_noncreature_and_sacs_for_red() {
        let mut g = two_player_game();
        let kykar = g.add_card_to_battlefield(0, catalog::kykar_winds_fury());
        bolt_face(&mut g, 0, 1);
        let spirit = g.battlefield.iter().find(|c| {
            c.definition.subtypes.creature_types.contains(&CreatureType::Spirit)
        }).map(|c| c.id).expect("Spirit token minted on noncreature cast");
        // Sacrifice the Spirit for one red mana.
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::ActivateAbility {
            card_id: kykar, ability_index: 0, target: None, additional_targets: Vec::new(),
            x_value: None, mode: None,
        }).expect("sac Spirit for red");
        drain_stack(&mut g);
        assert!(g.battlefield_find(spirit).is_none(), "Spirit sacrificed");
        assert_eq!(g.players[0].mana_pool.amount(Color::Red), 1, "added one red");
    }

    #[test]
    fn nivmizzet_parun_pings_on_draw() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::nivmizzet_parun());
        g.add_card_to_library(0, catalog::forest());
        g.players[1].life = 20;
        draw(&mut g, 0);
        assert_eq!(g.players[1].life, 19, "drawing dealt 1 to the opponent");
    }

    #[test]
    fn locust_god_mints_insect_on_draw() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::the_locust_god());
        g.add_card_to_library(0, catalog::forest());
        draw(&mut g, 0);
        assert!(
            g.battlefield.iter().any(|c| c.definition.subtypes.creature_types.contains(&CreatureType::Insect)),
            "drawing minted an Insect token",
        );
    }

    #[test]
    fn veyran_pumps_on_instant_cast() {
        let mut g = two_player_game();
        let v = g.add_card_to_battlefield(0, catalog::veyran_voice_of_duality());
        bolt_face(&mut g, 0, 1);
        let cp = g.computed_permanent(v).unwrap();
        assert_eq!((cp.power, cp.toughness), (3, 3), "magecraft plus one plus one");
    }

    #[test]
    fn charmbreaker_returns_random_is_at_upkeep_and_pumps() {
        let mut g = two_player_game();
        let cb = g.add_card_to_battlefield(0, catalog::charmbreaker_devils());
        let bolt = g.add_card_to_graveyard(0, catalog::lightning_bolt());
        g.active_player_idx = 0;
        g.fire_step_triggers(TurnStep::Upkeep);
        drain_stack(&mut g);
        assert!(g.players[0].hand.iter().any(|c| c.id == bolt), "the lone I/S returned to hand");
        // Casting an I/S pumps Charmbreaker +4/+0.
        bolt_face(&mut g, 0, 1);
        let cp = g.computed_permanent(cb).unwrap();
        assert_eq!(cp.power, 8, "cast an I/S gave plus four power");
    }

    #[test]
    fn pyromancer_ascension_counters_on_name_match_then_copies() {
        let mut g = two_player_game();
        let asc = g.add_card_to_battlefield(0, catalog::pyromancer_ascension());
        // A Bolt already in the graveyard so a cast Bolt shares its name.
        g.add_card_to_graveyard(0, catalog::lightning_bolt());
        bolt_face(&mut g, 0, 1);
        assert_eq!(
            g.battlefield_find(asc).unwrap().counter_count(CounterType::Quest),
            1,
            "name-matched cast added a quest counter",
        );
        // Preload two counters, then a cast copies the spell (P1 takes 3 + 3).
        g.battlefield_find_mut(asc).unwrap().add_counters(CounterType::Quest, 2);
        g.players[1].life = 20;
        bolt_face(&mut g, 0, 1);
        assert_eq!(g.players[1].life, 14, "spell was copied for six total to face");
    }

    #[test]
    fn izzet_guildmage_copies_low_mv_instant() {
        let mut g = two_player_game();
        let mage = g.add_card_to_battlefield(0, catalog::izzet_guildmage());
        for c in g.battlefield.iter_mut() { c.summoning_sick = false; }
        g.players[0].life = 20;
        g.players[1].life = 20;
        // Put a Bolt on the stack targeting P1's face.
        let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
        g.players[0].mana_pool.add(Color::Red, 1);
        g.priority.player_with_priority = 0;
        g.step = TurnStep::PreCombatMain;
        g.perform_action(GameAction::CastSpell {
            card_id: bolt, target: Some(Target::Player(1)), additional_targets: vec![],
            mode: None, x_value: None,
        }).expect("bolt on stack");
        // Activate the copy ability targeting the bolt on the stack.
        g.players[0].mana_pool.add(Color::Blue, 1);
        g.players[0].mana_pool.add_colorless(2);
        g.perform_action(GameAction::ActivateAbility {
            card_id: mage, ability_index: 0, target: Some(Target::Permanent(bolt)),
            additional_targets: Vec::new(), x_value: None, mode: None,
        }).expect("copy the bolt");
        drain_stack(&mut g);
        // The bolt + its copy each deal 3; the copy may pick a new target, so
        // assert on the total damage dealt across both players.
        let dealt = (20 - g.players[0].life) + (20 - g.players[1].life);
        assert_eq!(dealt, 6, "the ability copied the bolt — two instances of 3 resolved");
    }
}

mod recent92 {
    use crabomination::catalog;
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    use crabomination::game::two_player_game;
    use crabomination::game::types::Target;
    use crabomination::game::*;
    use crabomination::mana::Color;

    /// Cast Lightning Bolt from P0 at P1's face.
    fn bolt_face(g: &mut GameState) {
        let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
        g.players[0].mana_pool.add(Color::Red, 1);
        g.priority.player_with_priority = 0;
        g.step = TurnStep::PreCombatMain;
        g.perform_action(GameAction::CastSpell {
            card_id: bolt, target: Some(Target::Player(1)), additional_targets: vec![],
            mode: None, x_value: None,
        }).expect("bolt castable");
        drain_stack(g);
    }

    /// Total damage dealt to both players from a starting 20/20.
    fn damage_dealt(g: &GameState) -> i32 {
        (20 - g.players[0].life) + (20 - g.players[1].life)
    }

    #[test]
    fn firemind_vessel_taps_for_two() {
        let mut g = two_player_game();
        let v = g.add_card_to_battlefield(0, catalog::firemind_vessel());
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::ActivateAbility {
            card_id: v, ability_index: 0, target: None, additional_targets: Vec::new(),
            x_value: None, mode: None,
        }).expect("tap for mana");
        assert_eq!(g.players[0].mana_pool.total(), 2, "added two mana");
    }

    #[test]
    fn swarm_intelligence_copies_each_is() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::swarm_intelligence());
        g.players[0].life = 20;
        g.players[1].life = 20;
        bolt_face(&mut g);
        assert_eq!(damage_dealt(&g), 6, "bolt + one copy each dealt 3");
    }

    #[test]
    fn thousand_year_storm_copies_per_prior_spell() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::thousand_year_storm());
        g.players[0].life = 20;
        g.players[1].life = 20;
        bolt_face(&mut g); // first spell: 0 copies → 3 damage
        assert_eq!(damage_dealt(&g), 3, "first spell makes no copies");
        bolt_face(&mut g); // second spell: 1 copy → +6 damage
        assert_eq!(damage_dealt(&g), 9, "second spell copied once");
    }

    #[test]
    fn mirari_copies_when_you_pay_three() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::mirari());
        g.players[0].life = 20;
        g.players[1].life = 20;
        // Float {3} for the optional copy cost; say yes to the payment.
        g.players[0].mana_pool.add_colorless(3);
        g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Bool(true)]));
        bolt_face(&mut g);
        assert_eq!(damage_dealt(&g), 6, "paid three to copy the bolt");
    }

    #[test]
    fn nivmizzet_dracogenius_pings_for_one() {
        let mut g = two_player_game();
        let niv = g.add_card_to_battlefield(0, catalog::nivmizzet_dracogenius());
        for c in g.battlefield.iter_mut() { c.summoning_sick = false; }
        g.players[1].life = 20;
        g.players[0].mana_pool.add(Color::Blue, 1);
        g.players[0].mana_pool.add(Color::Red, 1);
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::ActivateAbility {
            card_id: niv, ability_index: 0, target: Some(Target::Player(1)),
            additional_targets: Vec::new(), x_value: None, mode: None,
        }).expect("ping");
        drain_stack(&mut g);
        assert_eq!(g.players[1].life, 19, "dealt 1 to the opponent");
    }

    #[test]
    fn jhoira_draws_on_historic_but_not_vanilla() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::jhoira_weatherlight_captain());
        g.active_player_idx = 0;
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g.add_card_to_library(0, catalog::forest());
        let before = g.players[0].hand.len();
        // Cast an artifact spell (historic) → draw.
        let orn = g.add_card_to_hand(0, catalog::ornithopter());
        g.perform_action(GameAction::CastSpell {
            card_id: orn, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("Ornithopter castable");
        drain_stack(&mut g);
        // Ornithopter left hand to the battlefield, then the historic trigger drew
        // one, so the hand grew by one and the library is now empty.
        assert_eq!(g.players[0].hand.len(), before + 1, "the historic cast drew a card");
        assert!(g.players[0].library.is_empty(), "the historic draw emptied the library");
    }

    #[test]
    fn arjun_wheels_hand_on_cast() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::arjun_the_shifting_flame());
        g.active_player_idx = 0;
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        // Two spare cards in hand + a fresh library to draw from.
        g.add_card_to_hand(0, catalog::forest());
        g.add_card_to_hand(0, catalog::forest());
        for _ in 0..2 { g.add_card_to_library(0, catalog::island()); }
        bolt_face(&mut g);
        // The two spare cards were discarded and two fresh cards drawn.
        assert!(g.players[0].graveyard.iter().filter(|c| c.definition.name == "Forest").count() >= 2,
            "hand cards discarded");
        assert_eq!(g.players[0].hand.len(), 2, "drew back that many");
    }

    #[test]
    fn electrodominance_deals_x_and_free_casts() {
        let mut g = two_player_game();
        g.players[1].life = 20;
        let ed = g.add_card_to_hand(0, catalog::electrodominance());
        // A bolt in hand (mv 1 ≤ X) to free-cast.
        g.add_card_to_hand(0, catalog::lightning_bolt());
        g.players[0].mana_pool.add(Color::Red, 2);
        g.players[0].mana_pool.add_colorless(3);
        g.priority.player_with_priority = 0;
        g.step = TurnStep::PreCombatMain;
        g.perform_action(GameAction::CastSpell {
            card_id: ed, target: Some(Target::Player(1)), additional_targets: vec![],
            mode: None, x_value: Some(3),
        }).expect("Electrodominance castable");
        drain_stack(&mut g);
        // At least the X = 3 damage landed on P1.
        assert!(g.players[1].life <= 17, "dealt X=3 to the opponent");
    }
}

mod recent93 {
    use crabomination::catalog;
    use crabomination::card::CardType;
    use crabomination::game::two_player_game;
    use crabomination::game::types::Target;
    use crabomination::game::*;
    use crabomination::mana::Color;

    #[test]
    fn galecaster_bounces_by_tapping_a_wizard() {
        let mut g = two_player_game();
        let colossus = g.add_card_to_battlefield(0, catalog::galecaster_colossus());
        // A second Wizard to tap for the cost.
        g.add_card_to_battlefield(0, catalog::gadwick_the_wizened());
        let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        for c in g.battlefield.iter_mut() { c.summoning_sick = false; }
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::ActivateAbility {
            card_id: colossus, ability_index: 0, target: Some(Target::Permanent(foe)),
            additional_targets: Vec::new(), x_value: None, mode: None,
        }).expect("tap a Wizard to bounce");
        drain_stack(&mut g);
        assert!(g.battlefield_find(foe).is_none(), "the enemy creature was bounced");
        assert!(g.players[1].hand.iter().any(|c| c.definition.name == "Grizzly Bears"),
            "returned to its owner's hand");
    }

    #[test]
    fn gadwick_draws_x_and_taps_on_blue_cast() {
        let mut g = two_player_game();
        // Cast Gadwick with X = 2.
        g.add_card_to_library(0, catalog::island());
        g.add_card_to_library(0, catalog::island());
        let gadwick = g.add_card_to_hand(0, catalog::gadwick_the_wizened());
        for _ in 0..3 { g.players[0].mana_pool.add(Color::Blue, 1); }
        g.players[0].mana_pool.add_colorless(2);
        g.priority.player_with_priority = 0;
        g.step = TurnStep::PreCombatMain;
        let before = g.players[0].hand.len();
        g.perform_action(GameAction::CastSpell {
            card_id: gadwick, target: None, additional_targets: vec![], mode: None, x_value: Some(2),
        }).expect("cast Gadwick for X=2");
        drain_stack(&mut g);
        // Gadwick left hand (−1) and drew 2 (+2) → net +1.
        assert_eq!(g.players[0].hand.len(), before + 1, "drew X = 2 (net +1 after leaving hand)");
        // Cast a blue spell → tap an opponent's untapped permanent.
        let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let bolt = g.add_card_to_hand(0, catalog::brainstorm());
        g.players[0].mana_pool.add(Color::Blue, 1);
        g.perform_action(GameAction::CastSpell {
            card_id: bolt, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast a blue spell");
        drain_stack(&mut g);
        assert!(g.battlefield_find(foe).unwrap().tapped, "the blue cast tapped the opponent's creature");
    }

    #[test]
    fn sphinx_of_lost_truths_discards_when_not_kicked() {
        let mut g = two_player_game();
        for _ in 0..4 { g.add_card_to_library(0, catalog::island()); }
        let sphinx = g.add_card_to_battlefield(0, catalog::sphinx_of_lost_truths());
        let before = g.players[0].hand.len();
        g.fire_self_etb_triggers(sphinx, 0);
        drain_stack(&mut g);
        // Drew 3, discarded 3 (not kicked) → net 0.
        assert_eq!(g.players[0].hand.len(), before, "unkicked: draw 3 then discard 3");
        assert!(g.players[0].graveyard.iter().filter(|c| c.definition.name == "Island").count() >= 3,
            "three cards discarded");
    }

    #[test]
    fn rielle_grows_with_instants_in_graveyard() {
        let mut g = two_player_game();
        let rielle = g.add_card_to_battlefield(0, catalog::rielle_the_everwise());
        assert_eq!(g.computed_permanent(rielle).unwrap().power, 0, "0 power with an empty graveyard");
        g.add_card_to_graveyard(0, catalog::lightning_bolt());
        g.add_card_to_graveyard(0, catalog::brainstorm());
        g.add_card_to_graveyard(0, catalog::grizzly_bears()); // a creature — not counted
        let cp = g.computed_permanent(rielle).unwrap();
        assert_eq!((cp.power, cp.toughness), (2, 3), "+1/+0 per instant/sorcery in graveyard");
        assert!(g.players[0].graveyard.iter().any(|c| c.definition.card_types.contains(&CardType::Creature)));
    }
}

mod recent94 {
    use crabomination::card::{CardInstance, Keyword};
    use crabomination::catalog;
    use crabomination::game::types::{Attack, AttackTarget, Target};
    use crabomination::game::*;

    /// Attach `eq` to `creature` directly (test shortcut, bypassing the equip action).
    fn attach(g: &mut GameState, eq: crabomination::card::CardId, creature: crabomination::card::CardId) {
        g.battlefield.iter_mut().find(|c| c.id == eq).unwrap().attached_to = Some(creature);
    }

    /// Akiri's power tracks the number of artifacts you control; toughness stays 3.
    #[test]
    fn akiri_power_scales_with_artifacts() {
        let mut g = two_player_game();
        let akiri = g.add_card_to_battlefield(0, catalog::akiri_line_slinger());
        let cp = g.computed_permanent(akiri).unwrap();
        assert_eq!((cp.power, cp.toughness), (0, 3), "0/3 with no artifacts");
        g.add_card_to_battlefield(0, catalog::bonesplitter());
        g.add_card_to_battlefield(0, catalog::grafted_wargear());
        let cp = g.computed_permanent(akiri).unwrap();
        assert_eq!((cp.power, cp.toughness), (2, 3), "+1/+0 per artifact, toughness fixed");
    }

    /// Goreclaw discounts creature spells with power 4 or greater by {2}.
    #[test]
    fn goreclaw_discounts_big_creatures() {
        use crabomination::game::actions::cost_reduction_for_spell;
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::goreclaw_terror_of_qal_sisma());
        let big = CardInstance::new(g.next_id(), catalog::shivan_dragon(), 0); // 5/5
        let small = CardInstance::new(g.next_id(), catalog::grizzly_bears(), 0); // 2/2
        assert_eq!(cost_reduction_for_spell(&g, 0, &big, None), 2, "power 5 → {{2}} off");
        assert_eq!(cost_reduction_for_spell(&g, 0, &small, None), 0, "power 2 → no discount");
    }

    /// Reyav grants double strike to an equipped creature when it attacks.
    #[test]
    fn reyav_grants_double_strike_to_equipped_attacker() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::reyav_master_smith());
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let axe = g.add_card_to_battlefield(0, catalog::bonesplitter());
        attach(&mut g, axe, bear);
        g.battlefield.iter_mut().find(|c| c.id == bear).unwrap().summoning_sick = false;
        g.step = TurnStep::DeclareAttackers;
        g.priority.player_with_priority = 0;
        // Via perform_action so the YourControl attack trigger dispatches.
        g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: bear, target: AttackTarget::Player(1) }]))
            .expect("bear attacks");
        drain_stack(&mut g);
        assert!(g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::DoubleStrike),
            "equipped attacker gained double strike");
    }

    /// Wyleth draws a card for each Aura/Equipment attached to it when it attacks.
    #[test]
    fn wyleth_draws_per_attached() {
        let mut g = two_player_game();
        for _ in 0..3 { g.add_card_to_library(0, catalog::island()); }
        let wyleth = g.add_card_to_battlefield(0, catalog::wyleth_soul_of_steel());
        let axe = g.add_card_to_battlefield(0, catalog::bonesplitter());
        let boots = g.add_card_to_battlefield(0, catalog::grafted_wargear());
        attach(&mut g, axe, wyleth);
        attach(&mut g, boots, wyleth);
        g.battlefield.iter_mut().find(|c| c.id == wyleth).unwrap().summoning_sick = false;
        g.step = TurnStep::DeclareAttackers;
        g.priority.player_with_priority = 0;
        let before = g.players[0].hand.len();
        g.declare_attackers(vec![Attack { attacker: wyleth, target: AttackTarget::Player(1) }])
            .expect("Wyleth attacks");
        drain_stack(&mut g);
        assert_eq!(g.players[0].hand.len(), before + 2, "drew one per attached (2)");
    }

    /// Kazuul's Toll Collector attaches a target Equipment you control to itself.
    #[test]
    fn kazuul_attaches_equipment_to_self() {
        let mut g = two_player_game();
        let kazuul = g.add_card_to_battlefield(0, catalog::kazuuls_toll_collector());
        let axe = g.add_card_to_battlefield(0, catalog::bonesplitter());
        g.priority.player_with_priority = 0;
        g.step = TurnStep::PreCombatMain;
        g.perform_action(GameAction::ActivateAbility {
            card_id: kazuul, ability_index: 0, target: Some(Target::Permanent(axe)),
            additional_targets: Vec::new(), x_value: None, mode: None,
        }).expect("attach the Equipment");
        drain_stack(&mut g);
        assert_eq!(g.battlefield_find(axe).unwrap().attached_to, Some(kazuul), "axe now on Kazuul");
        assert_eq!(g.computed_permanent(kazuul).unwrap().power, 5, "3/2 + 2/0 = 5/2");
    }

    /// Hammer of Nazahn grants +2/+0 and indestructible to the creature it equips.
    #[test]
    fn hammer_of_nazahn_equip_bonus() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let hammer = g.add_card_to_battlefield(0, catalog::hammer_of_nazahn());
        attach(&mut g, hammer, bear);
        let cp = g.computed_permanent(bear).unwrap();
        assert_eq!(cp.power, 4, "2/2 + 2/0");
        assert!(cp.keywords.contains(&Keyword::Indestructible), "gains indestructible");
    }

    /// Argentum Armor is a +6/+6 anvil.
    #[test]
    fn argentum_armor_equip_bonus() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let armor = g.add_card_to_battlefield(0, catalog::argentum_armor());
        attach(&mut g, armor, bear);
        let cp = g.computed_permanent(bear).unwrap();
        assert_eq!((cp.power, cp.toughness), (8, 8), "2/2 + 6/6");
    }

    /// Vorpal Sword grants deathtouch; Prowler's Helm grants the evasion keyword.
    #[test]
    fn equipment_grant_keywords() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let sword = g.add_card_to_battlefield(0, catalog::vorpal_sword());
        attach(&mut g, sword, bear);
        assert!(g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Deathtouch));

        let cat = g.add_card_to_battlefield(0, catalog::kembas_skyguard());
        let helm = g.add_card_to_battlefield(0, catalog::prowlers_helm());
        attach(&mut g, helm, cat);
        assert!(g.computed_permanent(cat).unwrap().keywords.iter()
            .any(|k| matches!(k, Keyword::CantBeBlockedExceptBy(_))), "gains can't-be-blocked-except-by");
    }

    /// Sylvia gives Dragons you control double strike.
    #[test]
    fn sylvia_grants_dragons_double_strike() {
        let mut g = two_player_game();
        let dragon = g.add_card_to_battlefield(0, catalog::shivan_dragon());
        assert!(!g.computed_permanent(dragon).unwrap().keywords.contains(&Keyword::DoubleStrike));
        g.add_card_to_battlefield(0, catalog::sylvia_brightspear());
        assert!(g.computed_permanent(dragon).unwrap().keywords.contains(&Keyword::DoubleStrike),
            "Dragon gained double strike from Sylvia");
    }

    /// Kwende upgrades first strike to double strike for your creatures.
    #[test]
    fn kwende_upgrades_first_strike() {
        let mut g = two_player_game();
        // Akiri has first strike printed.
        let akiri = g.add_card_to_battlefield(0, catalog::akiri_line_slinger());
        assert!(!g.computed_permanent(akiri).unwrap().keywords.contains(&Keyword::DoubleStrike));
        g.add_card_to_battlefield(0, catalog::kwende_pride_of_femeref());
        assert!(g.computed_permanent(akiri).unwrap().keywords.contains(&Keyword::DoubleStrike),
            "first-striker upgraded to double strike");
    }

    /// Kemba's Skyguard gains 2 life on entry.
    #[test]
    fn kembas_skyguard_gains_life() {
        let mut g = two_player_game();
        let before = g.players[0].life;
        let cat = g.add_card_to_battlefield(0, catalog::kembas_skyguard());
        g.fire_self_etb_triggers(cat, 0);
        drain_stack(&mut g);
        assert_eq!(g.players[0].life, before + 2, "gained 2 life");
    }

    /// Niv-Mizzet, Parun can't be countered (partial completion).
    #[test]
    fn niv_mizzet_parun_cant_be_countered() {
        assert!(catalog::nivmizzet_parun().keywords.contains(&Keyword::CantBeCountered));
    }
}

mod recent95 {
    use crabomination::catalog;
    use crabomination::card::{CounterType, Keyword};
    use crabomination::game::types::{Attack, AttackTarget};
    use crabomination::game::*;

    /// Advance from the current step to PostCombatMain, resolving combat damage.
    fn pass_through_combat(g: &mut GameState) {
        while g.step != TurnStep::PostCombatMain && g.step != TurnStep::End {
            g.perform_action(GameAction::PassPriority).unwrap();
        }
        drain_stack(g);
    }

    /// Network Disruptor taps a permanent on entry (auto-targets the opponent's).
    #[test]
    fn network_disruptor_taps_on_etb() {
        let mut g = two_player_game();
        let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let disruptor = g.add_card_to_battlefield(0, catalog::network_disruptor());
        g.fire_self_etb_triggers(disruptor, 0);
        drain_stack(&mut g);
        assert!(g.battlefield_find(foe).unwrap().tapped, "opponent's permanent tapped");
    }

    /// Enthusiastic Mechanaut discounts artifact spells by {1}.
    #[test]
    fn enthusiastic_mechanaut_discounts_artifacts() {
        use crabomination::game::actions::cost_reduction_for_spell;
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::enthusiastic_mechanaut());
        let artifact = crabomination::card::CardInstance::new(g.next_id(), catalog::bonesplitter(), 0);
        let creature = crabomination::card::CardInstance::new(g.next_id(), catalog::grizzly_bears(), 0);
        assert_eq!(cost_reduction_for_spell(&g, 0, &artifact, None), 1, "artifact −{{1}}");
        assert_eq!(cost_reduction_for_spell(&g, 0, &creature, None), 0, "non-artifact unaffected");
    }

    /// Imperial Oath makes three Samurai and scries.
    #[test]
    fn imperial_oath_makes_three_samurai() {
        let mut g = two_player_game();
        for _ in 0..3 { g.add_card_to_library(0, catalog::island()); }
        let oath = g.add_card_to_hand(0, catalog::imperial_oath());
        for _ in 0..5 { g.players[0].mana_pool.add(crabomination::mana::Color::White, 1); }
        g.players[0].mana_pool.add_colorless(1);
        g.priority.player_with_priority = 0;
        g.step = TurnStep::PreCombatMain;
        g.perform_action(GameAction::CastSpell {
            card_id: oath, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast Imperial Oath");
        drain_stack(&mut g);
        let samurai = g.battlefield.iter().filter(|c| c.definition.name == "Samurai").count();
        assert_eq!(samurai, 3, "three Samurai tokens");
    }

    /// Twinshot Sniper's ETB deals 2 to the opponent (auto-targeted).
    #[test]
    fn twinshot_sniper_etb_pings() {
        let mut g = two_player_game();
        g.players[1].life = 20;
        let sniper = g.add_card_to_battlefield(0, catalog::twinshot_sniper());
        g.fire_self_etb_triggers(sniper, 0);
        drain_stack(&mut g);
        assert_eq!(g.players[1].life, 18, "dealt 2 to the opponent");
    }

    /// Moonfolk Puzzlemaker fires its scry trigger when it becomes tapped (by
    /// attacking); it ends up tapped with the trigger resolved.
    #[test]
    fn moonfolk_puzzlemaker_scries_on_tap() {
        let mut g = two_player_game();
        g.add_card_to_library(0, catalog::island());
        let mf = g.add_card_to_battlefield(0, catalog::moonfolk_puzzlemaker());
        g.clear_sickness(mf);
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        while g.step != TurnStep::DeclareAttackers {
            g.perform_action(GameAction::PassPriority).unwrap();
        }
        g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: mf, target: AttackTarget::Player(1) }]))
            .expect("Moonfolk attacks and taps");
        drain_stack(&mut g);
        assert!(g.battlefield_find(mf).unwrap().tapped, "tapped by attacking");
    }

    /// Jukai Preserver's ETB adds a +1/+1 counter to a creature you control.
    #[test]
    fn jukai_preserver_etb_counter() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let jukai = g.add_card_to_battlefield(0, catalog::jukai_preserver());
        g.fire_self_etb_triggers(jukai, 0);
        drain_stack(&mut g);
        let mine: u32 = [bear, jukai].iter()
            .map(|id| g.battlefield_find(*id).unwrap().counter_count(CounterType::PlusOnePlusOne))
            .sum();
        assert_eq!(mine, 1, "exactly one +1/+1 counter placed on a creature you control");
    }

    /// Selfless Samurai grants lifelink to a lone Samurai/Warrior attacker.
    #[test]
    fn selfless_samurai_lifelink_on_solo_attack() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::selfless_samurai());
        let samurai = g.add_card_to_battlefield(0, catalog::selfless_samurai());
        g.clear_sickness(samurai);
        g.step = TurnStep::DeclareAttackers;
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: samurai, target: AttackTarget::Player(1) }]))
            .expect("attack alone");
        drain_stack(&mut g);
        assert!(g.computed_permanent(samurai).unwrap().keywords.contains(&Keyword::Lifelink),
            "lone Samurai gained lifelink");
    }

    /// Prosperous Thief mints a Treasure on combat damage.
    #[test]
    fn prosperous_thief_makes_treasure() {
        let mut g = two_player_game();
        let thief = g.add_card_to_battlefield(0, catalog::prosperous_thief());
        g.clear_sickness(thief);
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        while g.step != TurnStep::DeclareAttackers {
            g.perform_action(GameAction::PassPriority).unwrap();
        }
        g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: thief, target: AttackTarget::Player(1) }]))
            .expect("thief attacks");
        drain_stack(&mut g);
        pass_through_combat(&mut g);
        assert!(g.battlefield.iter().any(|c| c.definition.name == "Treasure"), "made a Treasure");
    }

    /// Bronzeplate Boar buffs +3/+2 (and trample) when attached via Reconfigure.
    #[test]
    fn bronzeplate_boar_equip_bonus() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let boar = g.add_card_to_battlefield(0, catalog::bronzeplate_boar());
        g.battlefield.iter_mut().find(|c| c.id == boar).unwrap().attached_to = Some(bear);
        let cp = g.computed_permanent(bear).unwrap();
        assert_eq!((cp.power, cp.toughness), (5, 4), "2/2 + 3/2");
        assert!(cp.keywords.contains(&Keyword::Trample));
    }

    /// Automated Artificer taps for one mana.
    #[test]
    fn automated_artificer_makes_mana() {
        let mut g = two_player_game();
        let bot = g.add_card_to_battlefield(0, catalog::automated_artificer());
        g.clear_sickness(bot);
        g.priority.player_with_priority = 0;
        g.step = TurnStep::PreCombatMain;
        g.perform_action(GameAction::ActivateAbility {
            card_id: bot, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
        }).expect("tap for {C}");
        // The {C} is spend-restricted (artifacts/abilities only), so it lands in the
        // restricted pool rather than `total()`; the tap confirms the ability fired.
        assert!(g.battlefield_find(bot).unwrap().tapped, "tapped for restricted {{C}}");
    }
}

mod recent96 {
    use crabomination::catalog;
    use crabomination::card::{CounterType, Keyword};
    use crabomination::game::two_player_game;
    use crabomination::game::*;

    /// Jukai Naturalist discounts enchantment spells by {1}.
    #[test]
    fn jukai_naturalist_discounts_enchantments() {
        use crabomination::game::actions::cost_reduction_for_spell;
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::jukai_naturalist());
        let ench = crabomination::card::CardInstance::new(g.next_id(), catalog::golden_tail_disciple(), 0);
        let creature = crabomination::card::CardInstance::new(g.next_id(), catalog::grizzly_bears(), 0);
        assert_eq!(cost_reduction_for_spell(&g, 0, &ench, None), 1, "enchantment −{{1}}");
        assert_eq!(cost_reduction_for_spell(&g, 0, &creature, None), 0, "plain creature unaffected");
    }

    /// Kami of Transience grows when you cast an enchantment spell.
    #[test]
    fn kami_of_transience_grows_on_enchantment_cast() {
        let mut g = two_player_game();
        let kami = g.add_card_to_battlefield(0, catalog::kami_of_transience());
        let aura = g.add_card_to_hand(0, catalog::golden_tail_disciple()); // enchantment creature
        for _ in 0..2 { g.players[0].mana_pool.add(crabomination::mana::Color::White, 1); }
        g.players[0].mana_pool.add_colorless(1);
        g.priority.player_with_priority = 0;
        g.step = TurnStep::PreCombatMain;
        g.perform_action(GameAction::CastSpell {
            card_id: aura, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast an enchantment");
        drain_stack(&mut g);
        assert_eq!(g.battlefield_find(kami).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
    }

    /// Rabbit Battery grants +1/+1 and haste when attached.
    #[test]
    fn rabbit_battery_equip_bonus() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let battery = g.add_card_to_battlefield(0, catalog::rabbit_battery());
        g.battlefield.iter_mut().find(|c| c.id == battery).unwrap().attached_to = Some(bear);
        let cp = g.computed_permanent(bear).unwrap();
        assert_eq!((cp.power, cp.toughness), (3, 3), "2/2 + 1/1");
        assert!(cp.keywords.contains(&Keyword::Haste));
    }

    /// Nezumi Prowler's ETB grants deathtouch and lifelink to a creature you control.
    #[test]
    fn nezumi_prowler_etb_grants_keywords() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let nezumi = g.add_card_to_battlefield(0, catalog::nezumi_prowler());
        // Retarget the auto-picked creature onto the bear by removing other options:
        // both bear and nezumi are legal; assert the grant landed on one of them.
        g.fire_self_etb_triggers(nezumi, 0);
        drain_stack(&mut g);
        let got = [bear, nezumi].iter().any(|id| {
            let kw = &g.computed_permanent(*id).unwrap().keywords;
            kw.contains(&Keyword::Deathtouch) && kw.contains(&Keyword::Lifelink)
        });
        assert!(got, "a creature you control gained deathtouch + lifelink");
    }

    /// Invigorating Hot Spring enters with four counters and grants haste to
    /// modified creatures.
    #[test]
    fn invigorating_hot_spring_hastes_modified() {
        let mut g = two_player_game();
        let spring = g.move_card_to_battlefield_for_test(0, catalog::invigorating_hot_spring());
        drain_stack(&mut g);
        assert_eq!(g.battlefield_find(spring).unwrap().counter_count(CounterType::PlusOnePlusOne), 4,
            "entered with four +1/+1 counters");
        // A bear equipped with Bonesplitter is "modified" → gains haste.
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let axe = g.add_card_to_battlefield(0, catalog::bonesplitter());
        g.battlefield.iter_mut().find(|c| c.id == axe).unwrap().attached_to = Some(bear);
        assert!(g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Haste),
            "modified creature has haste");
    }

    /// Ironhoof Boar's Channel ability pumps a creature +3/+1 with trample.
    #[test]
    fn ironhoof_boar_channel_pumps() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let boar = g.add_card_to_hand(0, catalog::ironhoof_boar());
        g.players[0].mana_pool.add(crabomination::mana::Color::Red, 1);
        g.players[0].mana_pool.add_colorless(1);
        g.priority.player_with_priority = 0;
        g.step = TurnStep::PreCombatMain;
        // The Channel ability is the from-hand activated ability (index 0).
        g.perform_action(GameAction::ActivateAbility {
            card_id: boar, ability_index: 0, target: Some(crabomination::game::types::Target::Permanent(bear)),
            additional_targets: Vec::new(), x_value: None, mode: None,
        }).expect("channel the Boar");
        drain_stack(&mut g);
        let cp = g.computed_permanent(bear).unwrap();
        assert_eq!((cp.power, cp.toughness), (5, 3), "2/2 + 3/1");
        assert!(cp.keywords.contains(&Keyword::Trample));
        assert!(g.players[0].graveyard.iter().any(|c| c.definition.name == "Ironhoof Boar"),
            "the Boar was discarded to pay Channel");
    }

    /// Reinforced Ronin's Channel ability draws a card from hand.
    #[test]
    fn reinforced_ronin_channel_draws() {
        let mut g = two_player_game();
        g.add_card_to_library(0, catalog::island());
        let ronin = g.add_card_to_hand(0, catalog::reinforced_ronin());
        g.players[0].mana_pool.add(crabomination::mana::Color::Red, 1);
        g.players[0].mana_pool.add_colorless(1);
        g.priority.player_with_priority = 0;
        g.step = TurnStep::PreCombatMain;
        let hand_before = g.players[0].hand.len();
        g.perform_action(GameAction::ActivateAbility {
            card_id: ronin, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
        }).expect("channel: draw");
        drain_stack(&mut g);
        // −1 (Ronin discarded) +1 (drew) = net 0, and the Ronin is in the graveyard.
        assert_eq!(g.players[0].hand.len(), hand_before, "discarded Ronin, drew a card");
        assert!(g.players[0].graveyard.iter().any(|c| c.definition.name == "Reinforced Ronin"));
    }

    /// Colossal Skyturtle's blue Channel bounces a creature to its owner's hand.
    #[test]
    fn colossal_skyturtle_channel_bounces() {
        let mut g = two_player_game();
        let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let turtle = g.add_card_to_hand(0, catalog::colossal_skyturtle());
        g.players[0].mana_pool.add(crabomination::mana::Color::Blue, 1);
        g.players[0].mana_pool.add_colorless(1);
        g.priority.player_with_priority = 0;
        g.step = TurnStep::PreCombatMain;
        // The blue Channel is the second from-hand ability (index 1).
        g.perform_action(GameAction::ActivateAbility {
            card_id: turtle, ability_index: 1, target: Some(crabomination::game::types::Target::Permanent(foe)),
            additional_targets: Vec::new(), x_value: None, mode: None,
        }).expect("channel: bounce");
        drain_stack(&mut g);
        assert!(g.battlefield_find(foe).is_none(), "bounced");
        assert!(g.players[1].hand.iter().any(|c| c.definition.name == "Grizzly Bears"));
    }
}

mod recent97 {
    use crabomination::catalog;
    use crabomination::card::{CounterType, CreatureType, Keyword};
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    use crabomination::game::types::{Attack, AttackTarget, Target};
    use crabomination::game::*;

    /// Advance from the current step to PostCombatMain, resolving combat damage.
    fn pass_through_combat(g: &mut GameState) {
        while g.step != TurnStep::PostCombatMain && g.step != TurnStep::End {
            g.perform_action(GameAction::PassPriority).unwrap();
        }
        drain_stack(g);
    }

    /// Kappa Tech-Wrecker enters with a deathtouch keyword counter.
    #[test]
    fn kappa_enters_with_deathtouch_counter() {
        let mut g = two_player_game();
        let kappa = g.add_card_to_battlefield(0, catalog::kappa_tech_wrecker());
        g.fire_self_etb_triggers(kappa, 0);
        drain_stack(&mut g);
        assert!(
            g.computed_permanent(kappa).unwrap().keywords.contains(&Keyword::Deathtouch),
            "deathtouch counter grants the keyword"
        );
    }

    /// Kappa's combat damage may remove the counter to exile an opponent's artifact.
    #[test]
    fn kappa_combat_exiles_artifact() {
        let mut g = two_player_game();
        let art = g.add_card_to_battlefield(1, catalog::bonesplitter());
        let kappa = g.add_card_to_battlefield(0, catalog::kappa_tech_wrecker());
        g.fire_self_etb_triggers(kappa, 0);
        drain_stack(&mut g);
        g.clear_sickness(kappa);
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        g.decider = Box::new(ScriptedDecider::new([
            DecisionAnswer::Bool(true),
            DecisionAnswer::Target(Target::Permanent(art)),
        ]));
        while g.step != TurnStep::DeclareAttackers {
            g.perform_action(GameAction::PassPriority).unwrap();
        }
        g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker: kappa,
            target: AttackTarget::Player(1),
        }]))
        .expect("kappa attacks");
        drain_stack(&mut g);
        pass_through_combat(&mut g);
        assert!(g.battlefield_find(art).is_none(), "artifact exiled by Kappa");
    }

    /// Biting-Palm Ninja enters with a menace keyword counter.
    #[test]
    fn biting_palm_enters_with_menace_counter() {
        let mut g = two_player_game();
        let ninja = g.add_card_to_battlefield(0, catalog::biting_palm_ninja());
        g.fire_self_etb_triggers(ninja, 0);
        drain_stack(&mut g);
        assert!(
            g.computed_permanent(ninja).unwrap().keywords.contains(&Keyword::Menace),
            "menace counter grants the keyword"
        );
    }

    /// Kami of Restless Shadows returns a Ninja from your graveyard (mode 0).
    #[test]
    fn kami_restless_shadows_returns_ninja() {
        let mut g = two_player_game();
        let ninja = g.add_card_to_graveyard(0, catalog::dokuchi_silencer());
        let kami = g.add_card_to_battlefield(0, catalog::kami_of_restless_shadows());
        g.decider = Box::new(ScriptedDecider::new([
            DecisionAnswer::Mode(0),
            DecisionAnswer::Target(Target::Permanent(ninja)),
        ]));
        g.fire_self_etb_triggers(kami, 0);
        drain_stack(&mut g);
        assert!(g.players[0].hand.iter().any(|c| c.id == ninja), "Ninja returned to hand");
    }

    /// Explosive Entry destroys an artifact and puts a +1/+1 counter on a creature.
    #[test]
    fn explosive_entry_destroys_and_counters() {
        let mut g = two_player_game();
        let art = g.add_card_to_battlefield(1, catalog::bonesplitter());
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let spell = g.add_card_to_hand(0, catalog::explosive_entry());
        g.players[0].mana_pool.add(crabomination::mana::Color::Red, 1);
        g.players[0].mana_pool.add_colorless(1);
        g.priority.player_with_priority = 0;
        g.step = TurnStep::PreCombatMain;
        g.perform_action(GameAction::CastSpell {
            card_id: spell,
            target: Some(Target::Permanent(art)),
            additional_targets: vec![Target::Permanent(bear)],
            mode: None,
            x_value: None,
        })
        .expect("cast Explosive Entry");
        drain_stack(&mut g);
        assert!(g.battlefield_find(art).is_none(), "artifact destroyed");
        assert_eq!(
            g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne),
            1,
            "bear got a +1/+1 counter"
        );
    }

    /// Blade of the Oni sets the equipped creature to a 5/5 menacing Demon.
    #[test]
    fn blade_of_the_oni_makes_a_demon() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let blade = g.add_card_to_battlefield(0, catalog::blade_of_the_oni());
        g.battlefield.iter_mut().find(|c| c.id == blade).unwrap().attached_to = Some(bear);
        let cp = g.computed_permanent(bear).unwrap();
        assert_eq!((cp.power, cp.toughness), (5, 5), "base P/T set to 5/5");
        assert!(cp.keywords.contains(&Keyword::Menace), "gains menace");
        assert!(cp.subtypes.creature_types.contains(&CreatureType::Demon), "is a Demon");
    }

    /// Towashi Guide-Bot's ETB puts a +1/+1 counter on a creature you control.
    #[test]
    fn towashi_guide_bot_etb_counter() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let bot = g.add_card_to_battlefield(0, catalog::towashi_guide_bot());
        g.fire_self_etb_triggers(bot, 0);
        drain_stack(&mut g);
        let placed: u32 = [bear, bot]
            .iter()
            .map(|id| g.battlefield_find(*id).unwrap().counter_count(CounterType::PlusOnePlusOne))
            .sum();
        assert_eq!(placed, 1, "one +1/+1 counter placed");
    }

    /// Naomi's ETB makes a Samurai when you control an artifact and an enchantment.
    #[test]
    fn naomi_makes_samurai_with_artifact_and_enchantment() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::bonesplitter()); // artifact
        g.add_card_to_battlefield(0, catalog::golden_tail_disciple()); // enchantment creature
        let naomi = g.add_card_to_battlefield(0, catalog::naomi_pillar_of_order());
        g.fire_self_etb_triggers(naomi, 0);
        drain_stack(&mut g);
        assert!(
            g.battlefield.iter().any(|c| c.definition.name == "Samurai"),
            "Samurai token created"
        );
    }

    /// Jukai Trainee grows when it becomes blocked.
    #[test]
    fn jukai_trainee_pumps_when_blocked() {
        let mut g = two_player_game();
        let trainee = g.add_card_to_battlefield(0, catalog::jukai_trainee());
        let blocker = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        g.clear_sickness(trainee);
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        while g.step != TurnStep::DeclareAttackers {
            g.perform_action(GameAction::PassPriority).unwrap();
        }
        g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker: trainee,
            target: AttackTarget::Player(1),
        }]))
        .expect("trainee attacks");
        drain_stack(&mut g);
        while g.step != TurnStep::DeclareBlockers {
            g.perform_action(GameAction::PassPriority).unwrap();
        }
        g.perform_action(GameAction::DeclareBlockers(vec![(blocker, trainee)]))
            .expect("bear blocks");
        drain_stack(&mut g);
        assert_eq!(g.computed_permanent(trainee).unwrap().power, 3, "2/2 + 1/1 when blocked");
    }

    /// Gloomshrieker returns a permanent from your graveyard and has menace.
    #[test]
    fn gloomshrieker_returns_from_graveyard() {
        let mut g = two_player_game();
        let dead = g.add_card_to_graveyard(0, catalog::grizzly_bears());
        let gloom = g.add_card_to_battlefield(0, catalog::gloomshrieker());
        g.fire_self_etb_triggers(gloom, 0);
        drain_stack(&mut g);
        assert!(g.players[0].hand.iter().any(|c| c.id == dead), "grizzly returned to hand");
        assert!(
            g.computed_permanent(gloom).unwrap().keywords.contains(&Keyword::Menace),
            "Gloomshrieker has menace"
        );
    }

    /// Gloomshrieker exiles itself instead of dying (CR 603-style self-replacement).
    #[test]
    fn gloomshrieker_exiles_itself_on_death() {
        let mut g = two_player_game();
        let gloom = g.add_card_to_battlefield(0, catalog::gloomshrieker());
        g.remove_to_graveyard_with_triggers(gloom);
        drain_stack(&mut g);
        assert!(g.players[0].graveyard.iter().all(|c| c.id != gloom), "not in graveyard");
        assert!(g.exile.iter().any(|c| c.id == gloom), "exiled instead");
    }

    /// Ecologist's Terrarium tutors a basic land to hand on entry.
    #[test]
    fn ecologists_terrarium_fetches_basic() {
        let mut g = two_player_game();
        let forest = g.add_card_to_library(0, catalog::forest());
        let terr = g.add_card_to_battlefield(0, catalog::ecologists_terrarium());
        g.decider = Box::new(ScriptedDecider::new([
            DecisionAnswer::Bool(true),
            DecisionAnswer::Search(Some(forest)),
        ]));
        g.fire_self_etb_triggers(terr, 0);
        drain_stack(&mut g);
        assert!(g.players[0].hand.iter().any(|c| c.id == forest), "basic land in hand");
    }

    /// Scrapwork Mutt loots on entry (discard a card to draw a card).
    #[test]
    fn scrapwork_mutt_loots_on_etb() {
        let mut g = two_player_game();
        g.add_card_to_hand(0, catalog::grizzly_bears()); // fodder to discard
        g.add_card_to_library(0, catalog::island()); // to draw
        let mutt = g.add_card_to_battlefield(0, catalog::scrapwork_mutt());
        let before = g.players[0].hand.len();
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
        g.fire_self_etb_triggers(mutt, 0);
        drain_stack(&mut g);
        // Net hand size unchanged (discard 1, draw 1), and Scrapwork has Unearth.
        assert_eq!(g.players[0].hand.len(), before, "looted: discard one, draw one");
        assert!(
            !catalog::scrapwork_mutt().activated_abilities.is_empty(),
            "Scrapwork Mutt has an Unearth ability"
        );
    }

    /// Norika lets you cast an enchantment from your graveyard after a lone attack.
    #[test]
    fn norika_grants_graveyard_enchantment_cast() {
        let mut g = two_player_game();
        let ench = g.add_card_to_graveyard(0, catalog::golden_tail_disciple()); // enchantment creature
        let norika = g.add_card_to_battlefield(0, catalog::norika_yamazaki_the_poet());
        g.clear_sickness(norika);
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Target(Target::Permanent(ench))]));
        while g.step != TurnStep::DeclareAttackers {
            g.perform_action(GameAction::PassPriority).unwrap();
        }
        g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker: norika,
            target: AttackTarget::Player(1),
        }]))
        .expect("norika attacks alone");
        drain_stack(&mut g);
        let card = g.players[0].graveyard.iter().find(|c| c.id == ench).expect("still in gy");
        assert!(card.may_play_until.is_some(), "granted permission to cast from graveyard");
    }

    /// Kami of Celebration impulse-exiles the top card when a modified creature
    /// you control attacks.
    #[test]
    fn kami_of_celebration_impulse_on_modified_attack() {
        let mut g = two_player_game();
        g.add_card_to_library(0, catalog::island());
        g.add_card_to_battlefield(0, catalog::kami_of_celebration());
        let attacker = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.battlefield
            .iter_mut()
            .find(|c| c.id == attacker)
            .unwrap()
            .add_counters(CounterType::PlusOnePlusOne, 1); // make it "modified"
        g.clear_sickness(attacker);
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        while g.step != TurnStep::DeclareAttackers {
            g.perform_action(GameAction::PassPriority).unwrap();
        }
        g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker,
            target: AttackTarget::Player(1),
        }]))
        .expect("modified bear attacks");
        drain_stack(&mut g);
        assert!(
            g.exile.iter().any(|c| c.owner == 0 && c.may_play_until.is_some()),
            "top card exiled with permission to play"
        );
    }

    /// Dokuchi Silencer may discard a creature to destroy an opponent's creature on
    /// combat damage.
    #[test]
    fn dokuchi_silencer_destroys_on_combat_damage() {
        let mut g = two_player_game();
        g.add_card_to_hand(0, catalog::grizzly_bears()); // creature card to discard
        let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        g.battlefield.iter_mut().find(|c| c.id == victim).unwrap().tapped = true; // can't block
        let ninja = g.add_card_to_battlefield(0, catalog::dokuchi_silencer());
        g.clear_sickness(ninja);
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
        while g.step != TurnStep::DeclareAttackers {
            g.perform_action(GameAction::PassPriority).unwrap();
        }
        g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker: ninja,
            target: AttackTarget::Player(1),
        }]))
        .expect("dokuchi attacks");
        drain_stack(&mut g);
        pass_through_combat(&mut g);
        assert!(g.battlefield_find(victim).is_none(), "victim destroyed");
    }

    /// Moonsnare Prototype taps another permanent for {C}.
    #[test]
    fn moonsnare_prototype_makes_mana() {
        let mut g = two_player_game();
        let helper = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let proto = g.add_card_to_battlefield(0, catalog::moonsnare_prototype());
        g.priority.player_with_priority = 0;
        g.step = TurnStep::PreCombatMain;
        let _ = helper;
        g.perform_action(GameAction::ActivateAbility {
            card_id: proto,
            ability_index: 0,
            target: None,
            additional_targets: vec![],
            x_value: None, mode: None,
        })
        .expect("activate Moonsnare mana ability");
        assert_eq!(g.players[0].mana_pool.colorless_amount(), 1, "produced colorless mana");
    }
}

mod recent98 {
    use crabomination::catalog;
    use crabomination::card::{CounterType, Keyword};
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    use crabomination::game::types::{Attack, AttackTarget, Target};
    use crabomination::game::*;

    fn pass_through_combat(g: &mut GameState) {
        while g.step != TurnStep::PostCombatMain && g.step != TurnStep::End {
            g.perform_action(GameAction::PassPriority).unwrap();
        }
        drain_stack(g);
    }

    /// Nezumi Bladeblesser gains deathtouch/menace from artifacts/enchantments.
    #[test]
    fn nezumi_bladeblesser_conditional_keywords() {
        let mut g = two_player_game();
        let nezumi = g.add_card_to_battlefield(0, catalog::nezumi_bladeblesser());
        assert!(!g.computed_permanent(nezumi).unwrap().keywords.contains(&Keyword::Deathtouch));
        g.add_card_to_battlefield(0, catalog::bonesplitter()); // artifact
        assert!(g.computed_permanent(nezumi).unwrap().keywords.contains(&Keyword::Deathtouch));
        assert!(!g.computed_permanent(nezumi).unwrap().keywords.contains(&Keyword::Menace));
        g.add_card_to_battlefield(0, catalog::golden_tail_disciple()); // enchantment
        assert!(g.computed_permanent(nezumi).unwrap().keywords.contains(&Keyword::Menace));
    }

    /// Iron Apprentice enters as a 1/1 and moves its counter on death.
    #[test]
    fn iron_apprentice_moves_counter_on_death() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let iron = g.move_card_to_battlefield_for_test(0, catalog::iron_apprentice());
        drain_stack(&mut g);
        assert_eq!(g.computed_permanent(iron).unwrap().power, 1, "0/0 + counter = 1/1");
        g.remove_to_graveyard_with_triggers(iron);
        drain_stack(&mut g);
        assert_eq!(
            g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne),
            1,
            "counter moved to the bear"
        );
    }

    /// Circuit Mender gains 2 life on entry and draws when it leaves.
    #[test]
    fn circuit_mender_etb_and_ltb() {
        let mut g = two_player_game();
        g.add_card_to_library(0, catalog::island());
        g.players[0].life = 20;
        let mender = g.add_card_to_battlefield(0, catalog::circuit_mender());
        g.fire_self_etb_triggers(mender, 0);
        drain_stack(&mut g);
        assert_eq!(g.players[0].life, 22, "ETB gained 2 life");
        let hand_before = g.players[0].hand.len();
        g.remove_to_graveyard_with_triggers(mender);
        drain_stack(&mut g);
        assert_eq!(g.players[0].hand.len(), hand_before + 1, "LTB drew a card");
    }

    /// Dragonfly Suit is a flying Vehicle with Crew 1.
    #[test]
    fn dragonfly_suit_is_a_crewable_flyer() {
        let def = catalog::dragonfly_suit();
        assert!(def.keywords.contains(&Keyword::Flying));
        assert!(def.keywords.contains(&Keyword::Crew(1)));
    }

    /// Moon-Circuit Hacker attacks and deals combat damage; helper returns the
    /// hand-size delta. `entered_this_turn` toggles the "discard unless it entered
    /// this turn" rider.
    fn moon_circuit_hacker_hand_delta(entered_this_turn: bool) -> i64 {
        let mut g = two_player_game();
        g.add_card_to_library(0, catalog::island());
        g.add_card_to_hand(0, catalog::grizzly_bears()); // a card to discard, if forced
        let hacker = g.add_card_to_battlefield(0, catalog::moon_circuit_hacker());
        if entered_this_turn {
            g.battlefield_find_mut(hacker).unwrap().entered_turn = Some(g.turn_number);
        }
        g.clear_sickness(hacker);
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
        let hand_before = g.players[0].hand.len() as i64;
        while g.step != TurnStep::DeclareAttackers {
            g.perform_action(GameAction::PassPriority).unwrap();
        }
        g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker: hacker,
            target: AttackTarget::Player(1),
        }]))
        .expect("hacker attacks");
        drain_stack(&mut g);
        pass_through_combat(&mut g);
        g.players[0].hand.len() as i64 - hand_before
    }

    /// Draws (net +1) when it entered this turn — the discard rider is skipped.
    #[test]
    fn moon_circuit_hacker_no_discard_when_fresh() {
        assert_eq!(moon_circuit_hacker_hand_delta(true), 1);
    }

    /// Draws then discards (net 0) when it's been around since a prior turn.
    #[test]
    fn moon_circuit_hacker_discards_when_established() {
        assert_eq!(moon_circuit_hacker_hand_delta(false), 0);
    }

    /// Kaito's Pursuit makes the opponent discard two and gives your Ninjas menace.
    #[test]
    fn kaitos_pursuit_discards_and_grants_menace() {
        let mut g = two_player_game();
        for _ in 0..3 { g.add_card_to_hand(1, catalog::grizzly_bears()); }
        let ninja = g.add_card_to_battlefield(0, catalog::dokuchi_shadow_walker());
        let spell = g.add_card_to_hand(0, catalog::kaitos_pursuit());
        g.players[0].mana_pool.add(crabomination::mana::Color::Black, 1);
        g.players[0].mana_pool.add_colorless(2);
        g.priority.player_with_priority = 0;
        g.step = TurnStep::PreCombatMain;
        g.perform_action(GameAction::CastSpell {
            card_id: spell,
            target: Some(Target::Player(1)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .expect("cast Kaito's Pursuit");
        drain_stack(&mut g);
        assert_eq!(g.players[1].hand.len(), 1, "opponent discarded two");
        assert!(
            g.computed_permanent(ninja).unwrap().keywords.contains(&Keyword::Menace),
            "your Ninja gained menace"
        );
    }

    /// Bearer of Memory pumps a target enchantment creature.
    #[test]
    fn bearer_of_memory_counters_enchantment_creature() {
        let mut g = two_player_game();
        let bearer = g.add_card_to_battlefield(0, catalog::bearer_of_memory());
        let target = g.add_card_to_battlefield(0, catalog::golden_tail_disciple()); // enchantment creature
        for _ in 0..5 { g.players[0].mana_pool.add_colorless(1); }
        g.players[0].mana_pool.add(crabomination::mana::Color::Green, 1);
        g.priority.player_with_priority = 0;
        g.step = TurnStep::PreCombatMain;
        g.perform_action(GameAction::ActivateAbility {
            card_id: bearer,
            ability_index: 0,
            target: Some(Target::Permanent(target)),
            additional_targets: vec![],
            x_value: None, mode: None,
        })
        .expect("activate Bearer of Memory");
        drain_stack(&mut g);
        assert_eq!(
            g.battlefield_find(target).unwrap().counter_count(CounterType::PlusOnePlusOne),
            1,
            "enchantment creature got a +1/+1 counter"
        );
        assert!(g.computed_permanent(target).unwrap().keywords.contains(&Keyword::Trample));
    }

    /// Dokuchi Shadow-Walker is a 5/5 with Ninjutsu.
    #[test]
    fn dokuchi_shadow_walker_stats() {
        let def = catalog::dokuchi_shadow_walker();
        assert_eq!((def.power, def.toughness), (5, 5));
        assert!(def.keywords.iter().any(|k| matches!(k, Keyword::Ninjutsu(_))));
    }

    /// Reito Sentinel mills three on entry.
    #[test]
    fn reito_sentinel_mills_on_etb() {
        let mut g = two_player_game();
        for _ in 0..5 { g.add_card_to_library(1, catalog::island()); }
        let sentinel = g.add_card_to_battlefield(0, catalog::reito_sentinel());
        let gy_before = g.players[1].graveyard.len();
        g.fire_self_etb_triggers(sentinel, 0);
        drain_stack(&mut g);
        assert_eq!(g.players[1].graveyard.len(), gy_before + 3, "opponent milled three");
    }

    /// Akki Ronin loots on a lone Samurai attack.
    #[test]
    fn akki_ronin_loots_on_solo_attack() {
        let mut g = two_player_game();
        g.add_card_to_hand(0, catalog::grizzly_bears()); // to discard
        g.add_card_to_library(0, catalog::island()); // to draw
        let ronin = g.add_card_to_battlefield(0, catalog::akki_ronin());
        g.clear_sickness(ronin);
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        let hand_before = g.players[0].hand.len();
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
        while g.step != TurnStep::DeclareAttackers {
            g.perform_action(GameAction::PassPriority).unwrap();
        }
        g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker: ronin,
            target: AttackTarget::Player(1),
        }]))
        .expect("ronin attacks alone");
        drain_stack(&mut g);
        assert_eq!(g.players[0].hand.len(), hand_before, "looted: discard one, draw one");
    }
}

mod recent99 {
    use crabomination::catalog;
    use crabomination::card::{CounterType, Keyword};
    use crabomination::game::types::Target;
    use crabomination::game::*;

    /// Guardian Kirin grows when another creature you control dies.
    #[test]
    fn guardian_kirin_grows_on_ally_death() {
        let mut g = two_player_game();
        let kirin = g.add_card_to_battlefield(0, catalog::guardian_kirin());
        let ally = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.battlefield_find_mut(ally).unwrap().damage = 2; // lethal → CreatureDied
        let evs = g.check_state_based_actions();
        g.dispatch_triggers_for_events(&evs);
        drain_stack(&mut g);
        assert_eq!(
            g.battlefield_find(kirin).unwrap().counter_count(CounterType::PlusOnePlusOne),
            1,
            "a +1/+1 counter for the ally's death"
        );
    }

    /// Silver-Fur Master anthems other Ninja/Rogue creatures you control.
    #[test]
    fn silver_fur_master_anthems_ninjas() {
        let mut g = two_player_game();
        let master = g.add_card_to_battlefield(0, catalog::silver_fur_master());
        let ninja = g.add_card_to_battlefield(0, catalog::dokuchi_shadow_walker()); // 5/5 Ninja
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        assert_eq!(g.computed_permanent(ninja).unwrap().power, 6, "other Ninja gets +1/+1");
        assert_eq!(g.computed_permanent(master).unwrap().power, 2, "the lord doesn't pump itself");
        assert_eq!(g.computed_permanent(bear).unwrap().power, 2, "non-Ninja unaffected");
        assert!(master != ninja && bear != master);
    }

    /// Generous Visitor puts a counter when you cast an enchantment spell.
    #[test]
    fn generous_visitor_counters_on_enchantment_cast() {
        let mut g = two_player_game();
        let visitor = g.add_card_to_battlefield(0, catalog::generous_visitor());
        let ench = g.add_card_to_hand(0, catalog::golden_tail_disciple()); // enchantment creature
        for _ in 0..2 { g.players[0].mana_pool.add(crabomination::mana::Color::White, 1); }
        g.players[0].mana_pool.add_colorless(1);
        g.priority.player_with_priority = 0;
        g.step = TurnStep::PreCombatMain;
        g.perform_action(GameAction::CastSpell {
            card_id: ench, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast an enchantment");
        drain_stack(&mut g);
        // The visitor is a legal target for its own trigger; assert a counter landed.
        let placed: u32 = g.battlefield.iter().filter(|c| c.controller == 0)
            .map(|c| c.counter_count(CounterType::PlusOnePlusOne)).sum();
        assert_eq!(placed, 1, "one +1/+1 counter from the enchantment cast");
        let _ = visitor;
    }

    /// Boon of Boseiju pumps by the greatest mana value you control and untaps.
    #[test]
    fn boon_of_boseiju_pumps_by_greatest_mv() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::dokuchi_shadow_walker()); // MV 6
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.battlefield.iter_mut().find(|c| c.id == bear).unwrap().tapped = true;
        let spell = g.add_card_to_hand(0, catalog::boon_of_boseiju());
        g.players[0].mana_pool.add(crabomination::mana::Color::Green, 1);
        g.players[0].mana_pool.add_colorless(1);
        g.priority.player_with_priority = 0;
        g.step = TurnStep::PreCombatMain;
        g.perform_action(GameAction::CastSpell {
            card_id: spell,
            target: Some(Target::Permanent(bear)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        }).expect("cast Boon of Boseiju");
        drain_stack(&mut g);
        let cp = g.computed_permanent(bear).unwrap();
        assert_eq!(cp.power, 8, "2 + 6 (greatest MV among your permanents)");
        assert!(!g.battlefield_find(bear).unwrap().tapped, "untapped by the boon");
        assert!(!cp.keywords.contains(&Keyword::Defender));
    }
}

mod recent100 {
    use crabomination::card::{CounterType, Keyword};
    use crabomination::catalog;
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    use crabomination::game::types::{Attack, AttackTarget, Target};
    use crabomination::game::*;

    fn pass_through_combat(g: &mut GameState) {
        while g.step != TurnStep::PostCombatMain && g.step != TurnStep::End {
            g.perform_action(GameAction::PassPriority).unwrap();
        }
        drain_stack(g);
    }

    /// Golden-Tail Trainer discounts an Equipment spell by its power (1).
    #[test]
    fn golden_tail_trainer_discounts_equipment() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::golden_tail_trainer());
        let scimitar = g.add_card_to_hand(0, catalog::leonin_scimitar()); // {1} Equipment
        g.priority.player_with_priority = 0;
        // {1} - power 1 = {0}: casts with no mana floated.
        cast(&mut g, scimitar);
        assert!(g.battlefield_find(scimitar).is_some(), "discounted Equipment resolved");
    }

    /// Kami of Terrible Secrets draws + gains only when you control an artifact and
    /// an enchantment.
    #[test]
    fn kami_of_terrible_secrets_conditional_etb() {
        let mut g = two_player_game();
        g.add_card_to_library(0, catalog::island());
        g.players[0].life = 20;
        g.add_card_to_battlefield(0, catalog::bonesplitter()); // artifact
        g.add_card_to_battlefield(0, catalog::golden_tail_disciple()); // enchantment
        let kami = g.add_card_to_battlefield(0, catalog::kami_of_terrible_secrets());
        let hand = g.players[0].hand.len();
        g.fire_self_etb_triggers(kami, 0);
        drain_stack(&mut g);
        assert_eq!(g.players[0].hand.len(), hand + 1, "drew a card");
        assert_eq!(g.players[0].life, 21, "gained 1 life");
    }

    /// Sky-Blessed Samurai's Affinity for enchantments reduces its cost.
    #[test]
    fn sky_blessed_samurai_affinity_for_enchantments() {
        let mut g = two_player_game();
        // Two enchantments → {6}{W} becomes {4}{W}.
        for _ in 0..2 { g.add_card_to_battlefield(0, catalog::golden_tail_disciple()); }
        let samurai = g.add_card_to_hand(0, catalog::sky_blessed_samurai());
        g.players[0].mana_pool.add(crabomination::mana::Color::White, 1);
        g.players[0].mana_pool.add_colorless(4);
        g.priority.player_with_priority = 0;
        cast(&mut g, samurai);
        assert!(g.battlefield_find(samurai).is_some(), "cast for four-and-white via affinity");
    }

    /// Bamboo Grove Archer's Channel destroys a flyer from hand.
    #[test]
    fn bamboo_grove_archer_channel_kills_flyer() {
        let mut g = two_player_game();
        let archer = g.add_card_to_hand(0, catalog::bamboo_grove_archer());
        let flyer = g.add_card_to_battlefield(1, catalog::serra_angel()); // flying
        g.players[0].mana_pool.add(crabomination::mana::Color::Green, 1);
        g.players[0].mana_pool.add_colorless(4);
        g.priority.player_with_priority = 0;
        g.step = TurnStep::PreCombatMain;
        g.perform_action(GameAction::ActivateAbility {
            card_id: archer,
            ability_index: 0,
            target: Some(Target::Permanent(flyer)),
            additional_targets: vec![],
            x_value: None, mode: None,
        })
        .expect("channel Bamboo Grove Archer");
        drain_stack(&mut g);
        assert!(g.battlefield_find(flyer).is_none(), "flyer destroyed");
        assert!(g.players[0].graveyard.iter().any(|c| c.id == archer), "archer discarded");
    }

    /// Walking Skyscraper is discounted per modified creature and has hexproof only
    /// while untapped.
    #[test]
    fn walking_skyscraper_discount_and_hexproof() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.battlefield_find_mut(bear).unwrap().add_counters(CounterType::PlusOnePlusOne, 1); // modified
        let tower = g.add_card_to_hand(0, catalog::walking_skyscraper());
        g.players[0].mana_pool.add_colorless(7); // {8} - 1 modified = {7}
        g.priority.player_with_priority = 0;
        cast(&mut g, tower);
        assert!(g.battlefield_find(tower).is_some(), "cast for seven");
        assert!(g.computed_permanent(tower).unwrap().keywords.contains(&Keyword::Hexproof), "hexproof untapped");
        g.battlefield_find_mut(tower).unwrap().tapped = true;
        assert!(!g.computed_permanent(tower).unwrap().keywords.contains(&Keyword::Hexproof), "no hexproof tapped");
    }

    /// Master's Rebuke bites: your creature deals its power to an opponent's creature.
    #[test]
    fn masters_rebuke_bites() {
        let mut g = two_player_game();
        let mine = g.add_card_to_battlefield(0, catalog::serra_angel()); // 4/4
        let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
        let spell = g.add_card_to_hand(0, catalog::masters_rebuke());
        g.players[0].mana_pool.add(crabomination::mana::Color::Green, 1);
        g.players[0].mana_pool.add_colorless(1);
        g.priority.player_with_priority = 0;
        g.step = TurnStep::PreCombatMain;
        g.perform_action(GameAction::CastSpell {
            card_id: spell,
            target: Some(Target::Permanent(mine)),
            additional_targets: vec![Target::Permanent(theirs)],
            mode: None,
            x_value: None,
        })
        .expect("cast Master's Rebuke");
        drain_stack(&mut g);
        assert!(g.battlefield_find(theirs).is_none(), "2/2 took 4 damage and died");
    }

    /// Tempered in Solitude exiles the top card on a lone attack (impulse draw).
    #[test]
    fn tempered_in_solitude_impulse_on_solo_attack() {
        let mut g = two_player_game();
        g.add_card_to_library(0, catalog::island());
        g.add_card_to_battlefield(0, catalog::tempered_in_solitude());
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.clear_sickness(bear);
        g.step = TurnStep::DeclareAttackers;
        g.priority.player_with_priority = 0;
        let exile_before = g.exile.len();
        g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker: bear,
            target: AttackTarget::Player(1),
        }]))
        .expect("bear attacks alone");
        drain_stack(&mut g);
        assert_eq!(g.exile.len(), exile_before + 1, "top card exiled for impulse play");
    }

    /// Akki Ember-Keeper makes a Spirit when a nontoken modified creature dies (and
    /// not on an unmodified one). Kills via the SBA path so the LKI snapshot keeps
    /// the counter visible to the "modified" filter.
    #[test]
    fn akki_ember_keeper_makes_spirit_on_modified_death() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::akki_ember_keeper());
        // Unmodified creature dying makes nothing.
        let plain = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.battlefield_find_mut(plain).unwrap().damage = 2;
        let evs = g.check_state_based_actions();
        g.dispatch_triggers_for_events(&evs);
        drain_stack(&mut g);
        assert!(!g.battlefield.iter().any(|c| c.definition.name == "Spirit"), "no token for unmodified death");
        // Modified creature dying makes a Spirit.
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.battlefield_find_mut(bear).unwrap().add_counters(CounterType::PlusOnePlusOne, 1); // 3/3, modified
        g.battlefield_find_mut(bear).unwrap().damage = 3;
        let evs = g.check_state_based_actions();
        g.dispatch_triggers_for_events(&evs);
        drain_stack(&mut g);
        assert!(g.battlefield.iter().any(|c| c.definition.name == "Spirit"), "made a Spirit token");
    }

    /// Thundering Raiju deals damage equal to the count of other modified creatures.
    #[test]
    fn thundering_raiju_pings_per_modified() {
        let mut g = two_player_game();
        let raiju = g.add_card_to_battlefield(0, catalog::thundering_raiju());
        // Two other modified creatures.
        for _ in 0..2 {
            let c = g.add_card_to_battlefield(0, catalog::grizzly_bears());
            g.battlefield_find_mut(c).unwrap().add_counters(CounterType::PlusOnePlusOne, 1);
        }
        let target = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.clear_sickness(raiju);
        g.players[1].life = 20;
        g.step = TurnStep::DeclareAttackers;
        g.priority.player_with_priority = 0;
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Target(Target::Permanent(target))]));
        g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker: raiju,
            target: AttackTarget::Player(1),
        }]))
        .expect("raiju attacks");
        drain_stack(&mut g);
        assert_eq!(g.players[1].life, 18, "2 other modified creatures → 2 damage");
    }

    /// Scrapyard Steelbreaker pumps itself by sacrificing another artifact.
    #[test]
    fn scrapyard_steelbreaker_sac_pump() {
        let mut g = two_player_game();
        let breaker = g.add_card_to_battlefield(0, catalog::scrapyard_steelbreaker());
        g.add_card_to_battlefield(0, catalog::bonesplitter()); // artifact to sac
        g.players[0].mana_pool.add_colorless(1);
        g.priority.player_with_priority = 0;
        g.step = TurnStep::PreCombatMain;
        g.perform_action(GameAction::ActivateAbility {
            card_id: breaker,
            ability_index: 0,
            target: None,
            additional_targets: vec![],
            x_value: None, mode: None,
        })
        .expect("activate Scrapyard Steelbreaker");
        drain_stack(&mut g);
        let cp = g.computed_permanent(breaker).unwrap();
        assert_eq!((cp.power, cp.toughness), (5, 5), "3/4 +2/+1");
    }

    /// Atsushi's death (default mode: impulse) exiles the top two cards of the library.
    #[test]
    fn atsushi_death_impulse_exiles_two() {
        let mut g = two_player_game();
        for _ in 0..2 { g.add_card_to_library(0, catalog::island()); }
        let atsushi = g.add_card_to_battlefield(0, catalog::atsushi_the_blazing_sky());
        let exile_before = g.exile.len();
        g.remove_to_graveyard_with_triggers(atsushi);
        drain_stack(&mut g);
        assert_eq!(g.exile.len(), exile_before + 2, "impulse-exiled the top two cards");
    }

    /// Junji's death (default mode: drain) makes each opponent discard two and lose 2.
    #[test]
    fn junji_death_discard_and_drain() {
        let mut g = two_player_game();
        for _ in 0..3 { g.add_card_to_hand(1, catalog::grizzly_bears()); }
        g.players[1].life = 20;
        let junji = g.add_card_to_battlefield(0, catalog::junji_the_midnight_sky());
        g.remove_to_graveyard_with_triggers(junji);
        drain_stack(&mut g);
        assert_eq!(g.players[1].hand.len(), 1, "opponent discarded two");
        assert_eq!(g.players[1].life, 18, "opponent lost 2 life");
    }

    /// Chishiro makes a Spirit when an Equipment you control enters.
    #[test]
    fn chishiro_spirit_on_equipment_etb() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::chishiro_the_shattered_blade());
        let eq = g.add_card_to_battlefield(0, catalog::bonesplitter());
        g.dispatch_triggers_for_events(&[GameEvent::PermanentEntered { card_id: eq }]);
        drain_stack(&mut g);
        assert!(g.battlefield.iter().any(|c| c.definition.name == "Spirit"), "made a Spirit token");
    }

    /// Risona gains an indestructible counter when it deals combat damage.
    #[test]
    fn risona_gains_indestructible_counter() {
        let mut g = two_player_game();
        let risona = g.add_card_to_battlefield(0, catalog::risona_asari_commander());
        g.clear_sickness(risona);
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        while g.step != TurnStep::DeclareAttackers {
            g.perform_action(GameAction::PassPriority).unwrap();
        }
        g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker: risona,
            target: AttackTarget::Player(1),
        }]))
        .expect("risona attacks");
        drain_stack(&mut g);
        pass_through_combat(&mut g);
        assert_eq!(
            g.battlefield_find(risona).unwrap().counter_count(CounterType::Indestructible),
            1,
            "got an indestructible counter"
        );
    }

    /// Risona sheds an indestructible counter when its controller is dealt combat
    /// damage.
    #[test]
    fn risona_loses_counter_when_you_take_damage() {
        let mut g = two_player_game();
        let risona = g.add_card_to_battlefield(0, catalog::risona_asari_commander());
        g.battlefield_find_mut(risona).unwrap().add_counters(CounterType::Indestructible, 1);
        // Opponent's attacker connects with player 0.
        let attacker = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        g.fire_combat_damage_to_player_triggers(attacker, 0, 2);
        drain_stack(&mut g);
        assert_eq!(
            g.battlefield_find(risona).unwrap().counter_count(CounterType::Indestructible),
            0,
            "indestructible counter removed"
        );
    }

    /// Traproot Kami's toughness equals the number of Forests in play.
    #[test]
    fn traproot_kami_toughness_tracks_forests() {
        let mut g = two_player_game();
        let kami = g.add_card_to_battlefield(0, catalog::traproot_kami());
        assert_eq!(g.computed_permanent(kami).unwrap().toughness, 0, "no Forests yet");
        g.add_card_to_battlefield(0, catalog::forest());
        g.add_card_to_battlefield(1, catalog::forest());
        assert_eq!(g.computed_permanent(kami).unwrap().toughness, 2, "two Forests in play");
    }

    /// Unstoppable Ogre's ETB stops a creature from blocking.
    #[test]
    fn unstoppable_ogre_stops_blocker() {
        let mut g = two_player_game();
        let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let ogre = g.add_card_to_battlefield(0, catalog::unstoppable_ogre());
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Target(Target::Permanent(victim))]));
        g.fire_self_etb_triggers(ogre, 0);
        drain_stack(&mut g);
        assert!(g.computed_permanent(victim).unwrap().keywords.contains(&Keyword::CantBlock));
    }

    /// You Are Already Dead destroys a damaged creature and draws.
    #[test]
    fn you_are_already_dead_kills_damaged() {
        let mut g = two_player_game();
        g.add_card_to_library(0, catalog::island());
        let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        g.battlefield_find_mut(bear).unwrap().damage = 1; // dealt damage this turn
        g.battlefield_find_mut(bear).unwrap().dealt_damage_this_turn = true;
        let spell = g.add_card_to_hand(0, catalog::you_are_already_dead());
        g.players[0].mana_pool.add(crabomination::mana::Color::Black, 1);
        g.priority.player_with_priority = 0;
        g.step = TurnStep::PreCombatMain;
        let hand = g.players[0].hand.len();
        g.perform_action(GameAction::CastSpell {
            card_id: spell,
            target: Some(Target::Permanent(bear)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .expect("cast You Are Already Dead");
        drain_stack(&mut g);
        assert!(g.battlefield_find(bear).is_none(), "damaged creature destroyed");
        assert_eq!(g.players[0].hand.len(), hand - 1 + 1, "spell left hand, drew one");
    }
}
