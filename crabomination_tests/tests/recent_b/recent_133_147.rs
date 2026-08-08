//! Tests for recentN card batches 133-147 (merged from per-batch micro-files).

mod recent133 {
    use crabomination::card::{CardType, Keyword};
    use crabomination::catalog;
    use crabomination::game::types::Target;
    use crabomination::game::*;
    use crabomination::game::{drain_stack, two_player_game};
    use crabomination::mana::Color;

    fn role_on(g: &GameState, host: CardId, name: &str) -> bool {
        g.battlefield.iter().any(|c| c.attached_to == Some(host) && c.definition.name == name)
    }

    /// Rat Out shrinks a creature and leaves a Rat.
    #[test]
    fn rat_out_shrink_and_rat() {
        let mut g = two_player_game();
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        let enemy = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let spell = g.add_card_to_hand(0, catalog::rat_out());
        g.players[0].mana_pool.add(Color::Black, 1);
        g.perform_action(GameAction::CastSpell {
            card_id: spell,
            target: Some(Target::Permanent(enemy)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .expect("cast Rat Out");
        drain_stack(&mut g);
        let cp = g.computed_permanent(enemy).unwrap();
        assert_eq!((cp.power, cp.toughness), (1, 1), "-1/-1");
        assert!(g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Rat"));
    }

    /// Eriette's Whisper discards two and hangs a Wicked Role that drains on death.
    #[test]
    fn eriettes_whisper_discard_and_wicked_role() {
        let mut g = two_player_game();
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.players[1].hand.clear();
        g.add_card_to_hand(1, catalog::grizzly_bears());
        g.add_card_to_hand(1, catalog::grizzly_bears());
        let spell = g.add_card_to_hand(0, catalog::eriettes_whisper());
        g.players[0].mana_pool.add(Color::Black, 1);
        g.players[0].mana_pool.add_colorless(3);
        g.perform_action(GameAction::CastSpell {
            card_id: spell,
            target: Some(Target::Permanent(bear)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .expect("cast Eriette's Whisper");
        drain_stack(&mut g);
        assert_eq!(g.players[1].hand.len(), 0, "opponent discarded both cards");
        assert!(role_on(&g, bear, "Wicked"), "Wicked Role attached");
        // The Role draining on death.
        g.players[1].life = 20;
        let role = g.battlefield.iter().find(|c| c.definition.name == "Wicked").unwrap().id;
        let ctx = crabomination::game::effects::EffectContext::for_ability(role, 0, Some(Target::Permanent(role)));
        g.resolve_effect(
            &crabomination::effect::Effect::SacrificePermanent { what: crabomination::effect::Selector::Target(0) },
            &ctx,
        )
        .unwrap();
        g.dispatch_triggers_for_events(&[]);
        drain_stack(&mut g);
        assert_eq!(g.players[1].life, 19, "opponent loses 1 when the Wicked Role dies");
    }

    /// Edgewall Pack makes a Rat on entry.
    #[test]
    fn edgewall_pack_rat_on_etb() {
        let mut g = two_player_game();
        let pack = g.add_card_to_battlefield(0, catalog::edgewall_pack());
        g.fire_self_etb_triggers(pack, 0);
        drain_stack(&mut g);
        assert!(g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Rat"));
    }

    /// Spider Food destroys a flyer and makes a Food.
    #[test]
    fn spider_food_destroy_and_food() {
        let mut g = two_player_game();
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        let flyer = g.add_card_to_battlefield(1, catalog::serra_angel()); // flying
        let spell = g.add_card_to_hand(0, catalog::spider_food());
        g.players[0].mana_pool.add(Color::Green, 1);
        g.players[0].mana_pool.add_colorless(2);
        g.perform_action(GameAction::CastSpell {
            card_id: spell,
            target: Some(Target::Permanent(flyer)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .expect("cast Spider Food");
        drain_stack(&mut g);
        assert!(g.battlefield_find(flyer).is_none(), "flyer destroyed");
        assert!(g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Food"));
    }

    /// Cursed Courtier shrinks itself to 1/1 with a Cursed Role.
    #[test]
    fn cursed_courtier_self_cursed_role() {
        let mut g = two_player_game();
        let courtier = g.add_card_to_battlefield(0, catalog::cursed_courtier());
        g.fire_self_etb_triggers(courtier, 0);
        drain_stack(&mut g);
        let cp = g.computed_permanent(courtier).unwrap();
        assert_eq!((cp.power, cp.toughness), (1, 1), "Cursed Role makes it 1/1");
        assert!(cp.keywords.contains(&Keyword::Lifelink));
    }

    /// Dutiful Griffin returns itself from the graveyard for two enchantments.
    #[test]
    fn dutiful_griffin_graveyard_recursion() {
        let mut g = two_player_game();
        // Griffin in graveyard, two enchantments to sacrifice.
        let griffin = g.add_card_to_hand(0, catalog::dutiful_griffin());
        let i = g.players[0].hand.iter().position(|c| c.id == griffin).unwrap();
        let c = g.players[0].hand.remove(i);
        g.players[0].graveyard.push(c);
        g.add_card_to_battlefield(0, dummy_enchantment());
        g.add_card_to_battlefield(0, dummy_enchantment());
        g.players[0].mana_pool.add(Color::White, 1);
        g.players[0].mana_pool.add_colorless(2);
        g.priority.player_with_priority = 0;
        g.step = TurnStep::PreCombatMain;
        g.perform_action(GameAction::ActivateAbility {
            card_id: griffin,
            ability_index: 0,
            target: None,
            additional_targets: vec![],
            x_value: None, mode: None,
        })
        .expect("activate graveyard return");
        drain_stack(&mut g);
        assert!(g.players[0].hand.iter().any(|c| c.id == griffin), "Griffin back in hand");
    }

    /// Tuinvale Guide grows and gains lifelink under Celebration.
    #[test]
    fn tuinvale_guide_celebration() {
        let mut g = two_player_game();
        let guide = g.add_card_to_battlefield(0, catalog::tuinvale_guide());
        // No celebration yet → bare 2/3, no lifelink.
        let cp = g.computed_permanent(guide).unwrap();
        assert!(!cp.keywords.contains(&Keyword::Lifelink));
        // Two nonland permanents entered this turn → celebration active.
        g.players[0].nonland_permanents_entered_this_turn = 2;
        let cp = g.computed_permanent(guide).unwrap();
        assert_eq!((cp.power, cp.toughness), (3, 3), "+1/+0");
        assert!(cp.keywords.contains(&Keyword::Lifelink));
    }

    /// Candy Trail scrys on entry and cashes in for life + a card.
    #[test]
    fn candy_trail_scry_and_sac() {
        let mut g = two_player_game();
        for _ in 0..3 {
            g.add_card_to_library(0, catalog::grizzly_bears());
        }
        let trail = g.add_card_to_battlefield(0, catalog::candy_trail());
        g.fire_self_etb_triggers(trail, 0); // scry 2, no assert on order
        drain_stack(&mut g);
        g.players[0].mana_pool.add_colorless(2);
        g.priority.player_with_priority = 0;
        g.step = TurnStep::PreCombatMain;
        let life = g.players[0].life;
        let hand = g.players[0].hand.len();
        g.perform_action(GameAction::ActivateAbility {
            card_id: trail,
            ability_index: 0,
            target: None,
            additional_targets: vec![],
            x_value: None, mode: None,
        })
        .expect("cash in Candy Trail");
        drain_stack(&mut g);
        assert_eq!(g.players[0].life, life + 3, "gained 3 life");
        assert_eq!(g.players[0].hand.len(), hand + 1, "drew a card");
        assert!(g.battlefield_find(trail).is_none(), "artifact sacrificed");
    }

    fn dummy_enchantment() -> crabomination::card::CardDefinition {
        use crabomination::mana::{cost, generic};
        crabomination::card::CardDefinition {
            name: "Test Glyph",
            cost: cost(&[generic(2)]),
            card_types: vec![CardType::Enchantment],
            ..Default::default()
        }
    }
}

mod recent134 {
    use crabomination::card::{CounterType, EnchantmentSubtype, Keyword};
    use crabomination::catalog;
    use crabomination::game::types::Target;
    use crabomination::game::*;
    use crabomination::game::{drain_stack, two_player_game};
    use crabomination::mana::Color;

    fn cast_adventure(g: &mut GameState, id: CardId, target: Option<Target>) {
        g.perform_action(GameAction::CastAdventure {
            card_id: id,
            target,
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .expect("cast adventure");
        drain_stack(g);
    }

    /// Entry Denied bounces a small creature.
    #[test]
    fn entry_denied_bounces_small() {
        let mut g = two_player_game();
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // MV2
        let card = g.add_card_to_hand(0, catalog::belunas_gatekeeper());
        g.players[0].mana_pool.add(Color::Blue, 1);
        g.players[0].mana_pool.add_colorless(1);
        cast_adventure(&mut g, card, Some(Target::Permanent(bear)));
        assert!(g.battlefield_find(bear).is_none(), "bear bounced");
        assert!(g.players[1].hand.iter().any(|c| c.definition.name == "Grizzly Bears"));
    }

    /// Freeze in Place taps and puts three stun counters on a creature.
    #[test]
    fn freeze_in_place_taps_and_stuns() {
        let mut g = two_player_game();
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        let enemy = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        for _ in 0..2 { g.add_card_to_library(0, catalog::grizzly_bears()); }
        let spell = g.add_card_to_hand(0, catalog::freeze_in_place());
        g.players[0].mana_pool.add(Color::Blue, 1);
        g.players[0].mana_pool.add_colorless(1);
        g.perform_action(GameAction::CastSpell {
            card_id: spell,
            target: Some(Target::Permanent(enemy)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .expect("cast Freeze in Place");
        drain_stack(&mut g);
        let ec = g.battlefield_find(enemy).unwrap();
        assert!(ec.tapped, "tapped");
        assert_eq!(ec.counter_count(CounterType::Stun), 3, "three stun counters");
    }

    /// Succumb to the Cold taps and stuns two creatures.
    #[test]
    fn succumb_stuns_two() {
        let mut g = two_player_game();
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        let a = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let b = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let spell = g.add_card_to_hand(0, catalog::succumb_to_the_cold());
        g.players[0].mana_pool.add(Color::Blue, 1);
        g.players[0].mana_pool.add_colorless(2);
        g.perform_action(GameAction::CastSpell {
            card_id: spell,
            target: Some(Target::Permanent(a)),
            additional_targets: vec![Target::Permanent(b)],
            mode: None,
            x_value: None,
        })
        .expect("cast Succumb to the Cold");
        drain_stack(&mut g);
        assert_eq!(g.battlefield_find(a).unwrap().counter_count(CounterType::Stun), 1);
        assert_eq!(g.battlefield_find(b).unwrap().counter_count(CounterType::Stun), 1);
    }

    /// Beat a Path stops a creature from blocking.
    #[test]
    fn beat_a_path_cant_block() {
        let mut g = two_player_game();
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        let enemy = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let card = g.add_card_to_hand(0, catalog::bellowing_bruiser());
        g.players[0].mana_pool.add(Color::Red, 1);
        g.players[0].mana_pool.add_colorless(2);
        cast_adventure(&mut g, card, Some(Target::Permanent(enemy)));
        assert!(g.computed_permanent(enemy).unwrap().keywords.contains(&Keyword::CantBlock));
    }

    /// Gallant Pie-Wielder gains double strike under Celebration.
    #[test]
    fn gallant_pie_wielder_celebration() {
        let mut g = two_player_game();
        let g_id = g.add_card_to_battlefield(0, catalog::gallant_pie_wielder());
        assert!(!g.computed_permanent(g_id).unwrap().keywords.contains(&Keyword::DoubleStrike));
        g.players[0].nonland_permanents_entered_this_turn = 2;
        assert!(g.computed_permanent(g_id).unwrap().keywords.contains(&Keyword::DoubleStrike));
    }

    /// Woodland Acolyte draws on entry; Mend the Wilds recurs a graveyard permanent.
    #[test]
    fn woodland_acolyte_etb_and_mend() {
        let mut g = two_player_game();
        // ETB draw.
        let acolyte = g.add_card_to_battlefield(0, catalog::woodland_acolyte());
        g.add_card_to_library(0, catalog::grizzly_bears());
        let before = g.players[0].hand.len();
        g.fire_self_etb_triggers(acolyte, 0);
        drain_stack(&mut g);
        assert_eq!(g.players[0].hand.len(), before + 1, "ETB draw");
        // Mend the Wilds from a fresh copy: put a graveyard creature on top.
        let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
        let card = g.add_card_to_hand(0, catalog::woodland_acolyte());
        g.players[0].mana_pool.add(Color::Green, 1);
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        cast_adventure(&mut g, card, Some(Target::Permanent(bear)));
        assert!(g.players[0].library.first().map(|c| c.id) == Some(bear), "bear on top of library");
    }

    /// Stroke of Midnight destroys a permanent and gives its controller a Human.
    #[test]
    fn stroke_of_midnight_destroy_and_human() {
        let mut g = two_player_game();
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        let enemy = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let spell = g.add_card_to_hand(0, catalog::stroke_of_midnight());
        g.players[0].mana_pool.add(Color::White, 1);
        g.players[0].mana_pool.add_colorless(2);
        g.perform_action(GameAction::CastSpell {
            card_id: spell,
            target: Some(Target::Permanent(enemy)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .expect("cast Stroke of Midnight");
        drain_stack(&mut g);
        assert!(g.battlefield_find(enemy).is_none(), "destroyed");
        assert!(
            g.battlefield.iter().any(|c| c.controller == 1 && c.definition.name == "Human"),
            "opponent got the Human",
        );
    }

    /// Return Triumphant reanimates a small creature with a Young Hero Role.
    #[test]
    fn return_triumphant_reanimates_with_role() {
        let mut g = two_player_game();
        let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
        let spell = g.add_card_to_hand(0, catalog::return_triumphant());
        g.players[0].mana_pool.add(Color::White, 1);
        g.players[0].mana_pool.add_colorless(1);
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::CastSpell {
            card_id: spell,
            target: Some(Target::Permanent(bear)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .expect("cast Return Triumphant");
        drain_stack(&mut g);
        assert!(g.battlefield_find(bear).is_some(), "bear reanimated");
        assert!(
            g.battlefield.iter().any(|c| c.attached_to == Some(bear)
                && c.definition.subtypes.enchantment_subtypes.contains(&EnchantmentSubtype::Role)),
            "Young Hero Role attached",
        );
    }

    /// Price of Beauty hangs a Wicked Role that drains on death.
    #[test]
    fn price_of_beauty_wicked_role() {
        let mut g = two_player_game();
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let card = g.add_card_to_hand(0, catalog::conceited_witch());
        g.players[0].mana_pool.add(Color::Black, 1);
        cast_adventure(&mut g, card, Some(Target::Permanent(bear)));
        assert!(
            g.battlefield.iter().any(|c| c.attached_to == Some(bear) && c.definition.name == "Wicked"),
            "Wicked Role attached",
        );
    }

    /// Sugar Rush pumps and draws.
    #[test]
    fn sugar_rush_pump_and_draw() {
        let mut g = two_player_game();
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.add_card_to_library(0, catalog::grizzly_bears());
        let spell = g.add_card_to_hand(0, catalog::sugar_rush());
        g.players[0].mana_pool.add(Color::Black, 1);
        g.players[0].mana_pool.add_colorless(1);
        let hand = g.players[0].hand.len();
        g.perform_action(GameAction::CastSpell {
            card_id: spell,
            target: Some(Target::Permanent(bear)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .expect("cast Sugar Rush");
        drain_stack(&mut g);
        assert_eq!(g.computed_permanent(bear).unwrap().power, 5, "+3/+0");
        assert_eq!(g.players[0].hand.len(), hand, "cast one, drew one → net same");
    }
}

mod recent135 {
    use crabomination::card::{CounterType, Keyword};
    use crabomination::catalog;
    use crabomination::game::types::{AttackTarget, Target};
    use crabomination::game::*;
    use crabomination::game::{drain_stack, two_player_game};
    use crabomination::mana::Color;

    fn cast(g: &mut GameState, id: CardId, target: Option<Target>) {
        g.perform_action(GameAction::CastSpell {
            card_id: id,
            target,
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .expect("cast");
        drain_stack(g);
    }

    fn kill(g: &mut GameState, id: CardId) {
        let ctl = g.battlefield_find(id).unwrap().controller;
        let ctx = crabomination::game::effects::EffectContext::for_ability(id, ctl, Some(Target::Permanent(id)));
        g.resolve_effect(
            &crabomination::effect::Effect::SacrificePermanent { what: crabomination::effect::Selector::Target(0) },
            &ctx,
        )
        .unwrap();
        g.dispatch_triggers_for_events(&[]);
        drain_stack(g);
    }

    fn activate(g: &mut GameState, id: CardId, idx: usize, target: Option<Target>) {
        g.perform_action(GameAction::ActivateAbility {
            card_id: id,
            ability_index: idx,
            target,
            additional_targets: vec![],
            x_value: None, mode: None,
        })
        .expect("activate");
        drain_stack(g);
    }

    fn has_token_attached(g: &GameState, host: CardId, name: &str) -> bool {
        g.battlefield.iter().any(|c| c.attached_to == Some(host) && c.definition.name == name)
    }

    fn tokens_named(g: &GameState, name: &str) -> usize {
        g.battlefield.iter().filter(|c| c.definition.name == name).count()
    }

    /// Chancellor of Tales copies an Adventure spell (via `CastSpellIsAdventure`),
    /// but not a normal instant/sorcery.
    #[test]
    fn chancellor_copies_adventure_spell() {
        let mut g = two_player_game();
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g.add_card_to_battlefield(0, catalog::chancellor_of_tales());
        let target = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        // Puny Snack (Gingerbread Hunter's adventure): -2/-2. Two copies kill a 2/2.
        let gh = g.add_card_to_hand(0, catalog::gingerbread_hunter());
        g.players[0].mana_pool.add(Color::Black, 1);
        g.players[0].mana_pool.add_colorless(2);
        g.perform_action(GameAction::CastAdventure {
            card_id: gh,
            target: Some(Target::Permanent(target)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .expect("cast Puny Snack");
        drain_stack(&mut g);
        // Original -2/-2 plus the copy's -2/-2 = -4/-4 → the 2/2 dies.
        assert!(g.battlefield_find(target).is_none(), "adventure copied → -4/-4 killed the bear");
    }

    /// A normal (non-Adventure) spell does not trigger Chancellor of Tales.
    #[test]
    fn chancellor_ignores_non_adventure() {
        let mut g = two_player_game();
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g.add_card_to_battlefield(0, catalog::chancellor_of_tales());
        let target = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let bolt = g.add_card_to_hand(0, catalog::monstrous_rage()); // +2/+0, not an adventure
        g.players[0].mana_pool.add(Color::Red, 1);
        cast(&mut g, bolt, Some(Target::Permanent(target)));
        // Only one Monster Role (no copy), so no second Role token was minted.
        assert_eq!(tokens_named(&g, "Monster"), 1, "non-adventure spell isn't copied");
    }

    /// Asinine Antics puts a Cursed Role on each opponent creature
    /// (`CreateTokenAttachedToEach`), turning them all into 1/1s.
    #[test]
    fn asinine_antics_curses_each_opponent_creature() {
        let mut g = two_player_game();
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        let a = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let b = g.add_card_to_battlefield(1, catalog::serra_angel());
        let spell = g.add_card_to_hand(0, catalog::asinine_antics());
        g.players[0].mana_pool.add(Color::Blue, 2);
        g.players[0].mana_pool.add_colorless(2);
        cast(&mut g, spell, None);
        assert!(has_token_attached(&g, a, "Cursed"), "bear cursed");
        assert!(has_token_attached(&g, b, "Cursed"), "angel cursed");
        assert_eq!(g.computed_permanent(b).unwrap().power, 1, "Cursed Role sets the angel to 1/1");
    }

    /// The Young Hero Role's counter-on-attack only fires while toughness ≤ 3.
    #[test]
    fn young_hero_gate_blocks_big_creature() {
        let mut g = two_player_game();
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        // Serra Angel is 4/4 — over the toughness gate.
        let angel = g.add_card_to_battlefield(0, catalog::serra_angel());
        // Attach a Young Hero Role via Embereth Veteran's sac ability.
        let vet = g.add_card_to_battlefield(0, catalog::embereth_veteran());
        g.step = TurnStep::PreCombatMain;
        g.players[0].mana_pool.add_colorless(1);
        activate(&mut g, vet, 0, Some(Target::Permanent(angel)));
        assert!(has_token_attached(&g, angel, "Young Hero"), "angel has the Role");
        g.clear_sickness(angel);
        while g.step != TurnStep::DeclareAttackers {
            g.perform_action(GameAction::PassPriority).unwrap();
        }
        g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker: angel,
            target: AttackTarget::Player(1),
        }]))
        .expect("attack");
        drain_stack(&mut g);
        assert_eq!(
            g.battlefield_find(angel).unwrap().counters.get(&CounterType::PlusOnePlusOne).copied().unwrap_or(0),
            0,
            "toughness 4 > 3 → no Young Hero counter",
        );
    }

    /// Feed the Cauldron destroys a small creature and, on your turn, makes a Food.
    #[test]
    fn feed_the_cauldron_makes_food_on_your_turn() {
        let mut g = two_player_game();
        g.active_player_idx = 0;
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let spell = g.add_card_to_hand(0, catalog::feed_the_cauldron());
        g.players[0].mana_pool.add(Color::Black, 1);
        g.players[0].mana_pool.add_colorless(2);
        cast(&mut g, spell, Some(Target::Permanent(bear)));
        assert!(g.battlefield_find(bear).is_none(), "bear destroyed");
        assert_eq!(tokens_named(&g, "Food"), 1, "your turn → Food created");
    }

    /// Collector's Vault loots and makes a Treasure.
    #[test]
    fn collectors_vault_loots_and_treasures() {
        let mut g = two_player_game();
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        let vault = g.add_card_to_battlefield(0, catalog::collectors_vault());
        g.add_card_to_library(0, catalog::grizzly_bears());
        g.add_card_to_hand(0, catalog::grizzly_bears());
        g.players[0].mana_pool.add_colorless(2);
        g.clear_sickness(vault);
        activate(&mut g, vault, 0, None);
        assert_eq!(tokens_named(&g, "Treasure"), 1, "Treasure created");
    }

    /// Cooped Up stops its host attacking; the activated ability exiles it.
    #[test]
    fn cooped_up_locks_then_exiles() {
        let mut g = two_player_game();
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let aura = g.add_card_to_hand(0, catalog::cooped_up());
        g.players[0].mana_pool.add(Color::White, 1);
        g.players[0].mana_pool.add_colorless(1);
        cast(&mut g, aura, Some(Target::Permanent(bear)));
        assert!(
            g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::CantAttack),
            "enchanted creature can't attack",
        );
        let aura_id = g.battlefield.iter().find(|c| c.definition.name == "Cooped Up").unwrap().id;
        g.players[0].mana_pool.add(Color::White, 1);
        g.players[0].mana_pool.add_colorless(2);
        activate(&mut g, aura_id, 0, None);
        assert!(g.battlefield_find(bear).is_none(), "bear exiled");
    }

    /// Twisted Sewer-Witch makes a Rat, then a Wicked Role on every Rat it controls.
    #[test]
    fn twisted_sewer_witch_roles_all_rats() {
        let mut g = two_player_game();
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        let witch = g.add_card_to_hand(0, catalog::twisted_sewer_witch());
        g.players[0].mana_pool.add(Color::Black, 2);
        g.players[0].mana_pool.add_colorless(3);
        cast(&mut g, witch, None);
        assert_eq!(tokens_named(&g, "Rat"), 1, "one Rat token");
        assert_eq!(tokens_named(&g, "Wicked"), 1, "a Wicked Role on the Rat");
        let rat = g.battlefield.iter().find(|c| c.definition.name == "Rat").unwrap().id;
        assert!(has_token_attached(&g, rat, "Wicked"), "Role attached to the Rat");
    }

    /// Mintstrosity leaves a Food behind when it dies.
    #[test]
    fn mintstrosity_dies_to_food() {
        let mut g = two_player_game();
        g.step = TurnStep::PreCombatMain;
        let mint = g.add_card_to_battlefield(0, catalog::mintstrosity());
        kill(&mut g, mint);
        assert_eq!(tokens_named(&g, "Food"), 1, "death → Food");
    }

    /// Dream Spoilers only fires on the opponent's turn.
    #[test]
    fn dream_spoilers_fires_off_turn() {
        let mut g = two_player_game();
        // Opponent's turn (player 1 active); Dream Spoilers controller is player 0.
        g.active_player_idx = 1;
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g.add_card_to_battlefield(0, catalog::dream_spoilers());
        let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let trick = g.add_card_to_hand(0, catalog::leaping_ambush());
        let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.players[0].mana_pool.add(Color::Green, 1);
        // Cast an instant during the opponent's turn → trigger targets their bear.
        g.perform_action(GameAction::CastSpell {
            card_id: trick,
            target: Some(Target::Permanent(mine)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .expect("cast during opponent turn");
        drain_stack(&mut g);
        assert_eq!(
            g.computed_permanent(bear).unwrap().toughness,
            1,
            "opponent's bear got -1/-1 from Dream Spoilers",
        );
    }

    /// Elvish Archivist draws once when an enchantment enters (once-per-turn gate).
    #[test]
    fn elvish_archivist_draws_on_enchantment() {
        let mut g = two_player_game();
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g.add_card_to_battlefield(0, catalog::elvish_archivist());
        g.add_card_to_library(0, catalog::grizzly_bears());
        // Cast an enchantment (Hopeful Vigil) so it enters and fires the draw.
        let ench = g.add_card_to_hand(0, catalog::hopeful_vigil());
        let hand_before = g.players[0].hand.len();
        g.players[0].mana_pool.add(Color::White, 1);
        g.players[0].mana_pool.add_colorless(1);
        cast(&mut g, ench, None);
        // Cast consumed Hopeful Vigil (−1) but the trigger drew one (+1) → net even.
        assert_eq!(g.players[0].hand.len(), hand_before, "drew for the enchantment entering");
    }

    /// Eriette's Tempting Apple steals a creature until end of turn.
    #[test]
    fn tempting_apple_steals_creature() {
        let mut g = two_player_game();
        g.active_player_idx = 0;
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let apple = g.add_card_to_hand(0, catalog::eriettes_tempting_apple());
        g.players[0].mana_pool.add_colorless(4);
        cast(&mut g, apple, Some(Target::Permanent(bear)));
        assert_eq!(g.battlefield_find(bear).unwrap().controller, 0, "gained control of the bear");
        assert!(g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Haste), "and it has haste");
    }
}

mod recent136 {
    use crabomination::card::Keyword;
    use crabomination::catalog;
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    use crabomination::game::types::Target;
    use crabomination::game::*;
    use crabomination::game::{drain_stack, two_player_game};
    use crabomination::mana::Color;

    fn cast(g: &mut GameState, id: CardId, target: Option<Target>) {
        g.perform_action(GameAction::CastSpell {
            card_id: id,
            target,
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .expect("cast");
        drain_stack(g);
    }

    fn activate(g: &mut GameState, id: CardId, idx: usize, target: Option<Target>) {
        g.perform_action(GameAction::ActivateAbility {
            card_id: id,
            ability_index: idx,
            target,
            additional_targets: vec![],
            x_value: None, mode: None,
        })
        .expect("activate");
        drain_stack(g);
    }

    /// Royal Treatment grants hexproof and mints a Royal Role.
    #[test]
    fn royal_treatment_hexproof_and_role() {
        let mut g = two_player_game();
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let spell = g.add_card_to_hand(0, catalog::royal_treatment());
        g.players[0].mana_pool.add(Color::Green, 1);
        cast(&mut g, spell, Some(Target::Permanent(bear)));
        let cp = g.computed_permanent(bear).unwrap();
        assert!(cp.keywords.contains(&Keyword::Hexproof), "gained hexproof");
        assert_eq!(cp.power, 3, "Royal Role gives +1/+1");
        assert!(
            g.battlefield.iter().any(|c| c.attached_to == Some(bear) && c.definition.name == "Royal"),
            "Royal Role attached",
        );
    }

    /// Merfolk Coralsmith's {1} ability shifts +1/-1 until end of turn.
    #[test]
    fn merfolk_coralsmith_self_pump() {
        let mut g = two_player_game();
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        let fish = g.add_card_to_battlefield(0, catalog::merfolk_coralsmith());
        g.players[0].mana_pool.add_colorless(1);
        activate(&mut g, fish, 0, None);
        let cp = g.computed_permanent(fish).unwrap();
        assert_eq!((cp.power, cp.toughness), (3, 2), "+1/-1 until end of turn");
    }

    /// Living Lectern sacrifices to draw and mint a Sorcerer Role on another creature.
    #[test]
    fn living_lectern_draws_and_roles() {
        let mut g = two_player_game();
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        let lectern = g.add_card_to_battlefield(0, catalog::living_lectern());
        let ally = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.add_card_to_library(0, catalog::grizzly_bears());
        let hand_before = g.players[0].hand.len();
        g.players[0].mana_pool.add_colorless(1);
        g.clear_sickness(lectern);
        activate(&mut g, lectern, 0, Some(Target::Permanent(ally)));
        assert_eq!(g.players[0].hand.len(), hand_before + 1, "drew a card");
        assert!(g.battlefield_find(lectern).is_none(), "Lectern sacrificed");
        assert!(
            g.battlefield.iter().any(|c| c.attached_to == Some(ally) && c.definition.name == "Sorcerer"),
            "Sorcerer Role on the ally",
        );
    }

    /// Stingblade Assassin destroys a creature that was dealt damage this turn.
    #[test]
    fn stingblade_kills_damaged_creature() {
        let mut g = two_player_game();
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        let victim = g.add_card_to_battlefield(1, catalog::serra_angel());
        // Mark the angel as damaged this turn.
        let ctx = crabomination::game::effects::EffectContext::for_ability(
            victim, 0, Some(Target::Permanent(victim)),
        );
        g.resolve_effect(
            &crabomination::effect::Effect::DealDamage {
                to: crabomination::effect::Selector::Target(0),
                amount: crabomination::effect::Value::Const(1),
            },
            &ctx,
        )
        .unwrap();
        let assassin = g.add_card_to_hand(0, catalog::stingblade_assassin());
        g.players[0].mana_pool.add(Color::Black, 1);
        g.players[0].mana_pool.add_colorless(3);
        cast(&mut g, assassin, Some(Target::Permanent(victim)));
        assert!(g.battlefield_find(victim).is_none(), "damaged angel destroyed");
    }

    /// Lord Skitter's Butcher mode 0 makes a Rat token.
    #[test]
    fn lord_skitters_butcher_makes_rat() {
        let mut g = two_player_game();
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        // Force mode 0 (create a Rat).
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Mode(0)]));
        let butcher = g.add_card_to_hand(0, catalog::lord_skitters_butcher());
        g.players[0].mana_pool.add(Color::Black, 1);
        g.players[0].mana_pool.add_colorless(2);
        cast(&mut g, butcher, None);
        assert!(
            g.battlefield.iter().any(|c| c.definition.name == "Rat"),
            "mode 0 created a Rat token",
        );
    }

    /// Provisions Merchant enters with a Food token.
    #[test]
    fn provisions_merchant_makes_food() {
        let mut g = two_player_game();
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        let merch = g.add_card_to_hand(0, catalog::provisions_merchant());
        g.players[0].mana_pool.add(Color::Green, 2);
        g.players[0].mana_pool.add_colorless(2);
        cast(&mut g, merch, None);
        assert!(
            g.battlefield.iter().any(|c| c.definition.name == "Food"),
            "ETB created a Food token",
        );
    }

    /// Scarecrow Guide's once-per-turn mana ability can't be activated twice.
    #[test]
    fn scarecrow_guide_once_per_turn_mana() {
        let mut g = two_player_game();
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        let guide = g.add_card_to_battlefield(0, catalog::scarecrow_guide());
        g.clear_sickness(guide);
        g.players[0].mana_pool.add_colorless(2);
        activate(&mut g, guide, 0, None);
        // Second activation this turn must be rejected (once-per-turn gate).
        let err = g.perform_action(GameAction::ActivateAbility {
            card_id: guide,
            ability_index: 0,
            target: None,
            additional_targets: vec![],
            x_value: None, mode: None,
        });
        assert!(err.is_err(), "second activation blocked by once-per-turn");
    }
}

mod recent137 {
    use crabomination::catalog;
    use crabomination::game::types::Target;
    use crabomination::game::*;
    use crabomination::game::{drain_stack, two_player_game};
    use crabomination::mana::Color;

    fn cast(g: &mut GameState, id: CardId, target: Option<Target>) {
        g.perform_action(GameAction::CastSpell {
            card_id: id,
            target,
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .expect("cast");
        drain_stack(g);
    }

    fn cast_adventure(g: &mut GameState, id: CardId, target: Option<Target>) {
        g.perform_action(GameAction::CastAdventure {
            card_id: id,
            target,
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .expect("cast adventure");
        drain_stack(g);
    }

    /// Storyteller Pixie draws when you cast an Adventure spell.
    #[test]
    fn storyteller_pixie_draws_on_adventure() {
        let mut g = two_player_game();
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g.add_card_to_battlefield(0, catalog::storyteller_pixie());
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.add_card_to_library(0, catalog::grizzly_bears());
        // Ride the Rails (Minecart Daredevil's adventure): +2/+1.
        let daredevil = g.add_card_to_hand(0, catalog::minecart_daredevil());
        let hand_before = g.players[0].hand.len();
        g.players[0].mana_pool.add(Color::Red, 1);
        g.players[0].mana_pool.add_colorless(1);
        cast_adventure(&mut g, daredevil, Some(Target::Permanent(bear)));
        // Cast consumed the adventurer (−1) but the Pixie drew (+1) → net even.
        assert_eq!(g.players[0].hand.len(), hand_before, "Pixie drew for the Adventure cast");
    }

    /// Desperate Parry weakens a blocker with -4/-0.
    #[test]
    fn desperate_parry_shrinks_power() {
        let mut g = two_player_game();
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        let angel = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4
        let card = g.add_card_to_hand(0, catalog::obyras_attendants());
        g.players[0].mana_pool.add(Color::Blue, 1);
        g.players[0].mana_pool.add_colorless(1);
        cast_adventure(&mut g, card, Some(Target::Permanent(angel)));
        assert_eq!(g.computed_permanent(angel).unwrap().power, 0, "-4/-0 zeroed the angel's power");
    }

    /// High Fae Negotiator's bargained ETB drains for 3.
    #[test]
    fn high_fae_negotiator_bargain_drain() {
        let mut g = two_player_game();
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        // A Treasure token to sacrifice for Bargain.
        let treasure = g.add_token_to_battlefield(0, &crabomination_base::tokens::treasure_token());
        let card = g.add_card_to_hand(0, catalog::high_fae_negotiator());
        g.players[0].life = 20;
        g.players[1].life = 20;
        g.players[0].mana_pool.add(Color::Black, 2);
        g.players[0].mana_pool.add_colorless(3);
        g.perform_action(GameAction::CastSpellBargain {
            card_id: card,
            sacrifice: Some(treasure),
            target: None,
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .expect("cast bargained");
        drain_stack(&mut g);
        assert_eq!(g.players[1].life, 17, "opponent lost 3 to the bargained ETB");
        assert_eq!(g.players[0].life, 23, "you gained 3");
    }

    /// Fell Horseman's Deathly Ride returns a creature card from the graveyard.
    #[test]
    fn deathly_ride_returns_from_graveyard() {
        let mut g = two_player_game();
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        let dead = g.add_card_to_graveyard(0, catalog::grizzly_bears());
        let card = g.add_card_to_hand(0, catalog::fell_horseman());
        g.players[0].mana_pool.add(Color::Black, 1);
        g.players[0].mana_pool.add_colorless(1);
        cast_adventure(&mut g, card, Some(Target::Permanent(dead)));
        assert!(
            g.players[0].hand.iter().any(|c| c.definition.name == "Grizzly Bears"),
            "Grizzly Bears returned to hand",
        );
    }

    /// Shrouded Shepherd's ETB pumps a creature you control.
    #[test]
    fn shrouded_shepherd_etb_pump() {
        let mut g = two_player_game();
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let card = g.add_card_to_hand(0, catalog::shrouded_shepherd());
        g.players[0].mana_pool.add(Color::White, 1);
        g.players[0].mana_pool.add_colorless(1);
        cast(&mut g, card, Some(Target::Permanent(bear)));
        assert_eq!(g.computed_permanent(bear).unwrap().toughness, 4, "+2/+2 on the bear");
    }

    /// Intrepid Trufflesnout makes a Food only when it attacks alone.
    #[test]
    fn trufflesnout_food_on_solo_attack() {
        let mut g = two_player_game();
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        let boar = g.add_card_to_battlefield(0, catalog::intrepid_trufflesnout());
        g.step = TurnStep::PreCombatMain;
        g.clear_sickness(boar);
        while g.step != TurnStep::DeclareAttackers {
            g.perform_action(GameAction::PassPriority).unwrap();
        }
        g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker: boar,
            target: crabomination::game::types::AttackTarget::Player(1),
        }]))
        .expect("attack alone");
        drain_stack(&mut g);
        assert!(
            g.battlefield.iter().any(|c| c.definition.name == "Food"),
            "attacking alone made a Food",
        );
    }
}

mod recent138 {
    use crabomination::card::{CardInstance, Keyword};
    use crabomination::catalog;
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    use crabomination::game::types::Target;
    use crabomination::game::*;
    use crabomination::game::{drain_stack, two_player_game};
    use crabomination::mana::Color;

    fn cast(g: &mut GameState, id: CardId, target: Option<Target>, mode: Option<usize>) {
        g.perform_action(GameAction::CastSpell {
            card_id: id,
            target,
            additional_targets: vec![],
            mode,
            x_value: None,
        })
        .expect("cast");
        drain_stack(g);
    }

    fn activate(g: &mut GameState, id: CardId, idx: usize) {
        g.perform_action(GameAction::ActivateAbility {
            card_id: id,
            ability_index: idx,
            target: None,
            additional_targets: vec![],
            x_value: None, mode: None,
        })
        .expect("activate");
        drain_stack(g);
    }

    /// A Tale for the Ages gives +2/+2 to an enchanted creature you control, but not
    /// to an unenchanted one.
    #[test]
    fn a_tale_for_the_ages_anthems_enchanted() {
        let mut g = two_player_game();
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        let enchanted = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
        let plain = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
        // Hang a Royal Role (+1/+1) so `enchanted` is enchanted.
        let rt = g.add_card_to_hand(0, catalog::royal_treatment());
        g.players[0].mana_pool.add(Color::Green, 1);
        cast(&mut g, rt, Some(Target::Permanent(enchanted)), None);
        g.add_card_to_battlefield(0, catalog::a_tale_for_the_ages());
        // enchanted: 2/2 base + Royal Role +1/+1 + anthem +2/+2 = 5/5.
        let cp = g.computed_permanent(enchanted).unwrap();
        assert_eq!((cp.power, cp.toughness), (5, 5), "anthem hits the enchanted creature");
        // plain: no Aura → no anthem.
        let cp = g.computed_permanent(plain).unwrap();
        assert_eq!((cp.power, cp.toughness), (2, 2), "unenchanted creature unaffected");
    }

    /// Break the Spell draws when destroying an enchantment you control; not when
    /// destroying an opponent's nontoken enchantment.
    #[test]
    fn break_the_spell_conditional_draw() {
        let mut g = two_player_game();
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        let mine = g.add_card_to_battlefield(0, catalog::a_tale_for_the_ages());
        g.add_card_to_library(0, catalog::grizzly_bears());
        let spell = g.add_card_to_hand(0, catalog::break_the_spell());
        let before = g.players[0].hand.len(); // includes the spell itself
        g.players[0].mana_pool.add(Color::White, 1);
        cast(&mut g, spell, Some(Target::Permanent(mine)), None);
        assert!(g.battlefield_find(mine).is_none(), "own enchantment destroyed");
        // -1 for the spell leaving hand, +1 for the draw → net even.
        assert_eq!(g.players[0].hand.len(), before, "drew a card for own enchantment");

        // Opponent's nontoken enchantment → no draw.
        let theirs = g.add_card_to_battlefield(1, catalog::a_tale_for_the_ages());
        g.add_card_to_library(0, catalog::grizzly_bears());
        let spell = g.add_card_to_hand(0, catalog::break_the_spell());
        let before = g.players[0].hand.len();
        g.players[0].mana_pool.add(Color::White, 1);
        cast(&mut g, spell, Some(Target::Permanent(theirs)), None);
        assert!(g.battlefield_find(theirs).is_none(), "opponent enchantment destroyed");
        assert_eq!(g.players[0].hand.len(), before - 1, "no draw (not yours, not a token)");
    }

    /// Moment of Valor mode 2 destroys a creature with power 4 or greater.
    #[test]
    fn moment_of_valor_destroys_big() {
        let mut g = two_player_game();
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        let angel = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4
        let spell = g.add_card_to_hand(0, catalog::moment_of_valor());
        g.players[0].mana_pool.add(Color::White, 1);
        g.players[0].mana_pool.add_colorless(2);
        cast(&mut g, spell, Some(Target::Permanent(angel)), Some(1));
        assert!(g.battlefield_find(angel).is_none(), "power-4 creature destroyed");
    }

    /// Gruff Triplets makes two token copies of itself on entry (three total).
    #[test]
    fn gruff_triplets_self_copies() {
        let mut g = two_player_game();
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        let id = g.add_card_to_hand(0, catalog::gruff_triplets());
        g.players[0].mana_pool.add(Color::Green, 3);
        g.players[0].mana_pool.add_colorless(3);
        cast(&mut g, id, None, None);
        let count = g
            .battlefield
            .iter()
            .filter(|c| c.controller == 0 && c.definition.name == "Gruff Triplets")
            .count();
        assert_eq!(count, 3, "original plus two token copies");
    }

    /// Howling Galefang has haste only while you own an exiled Adventure card.
    #[test]
    fn howling_galefang_haste_from_exiled_adventure() {
        let mut g = two_player_game();
        let gale = g.add_card_to_battlefield(0, catalog::howling_galefang());
        assert!(
            !g.computed_permanent(gale).unwrap().keywords.contains(&Keyword::Haste),
            "no haste without an exiled Adventure",
        );
        let mut ex = CardInstance::new(g.next_id(), catalog::minecart_daredevil(), 0);
        ex.on_adventure = true;
        g.exile.push(ex);
        assert!(
            g.computed_permanent(gale).unwrap().keywords.contains(&Keyword::Haste),
            "haste while an owned Adventure waits in exile",
        );
    }

    /// Experimental Confectioner turns a sacrificed Food into a Rat.
    #[test]
    fn experimental_confectioner_food_to_rat() {
        let mut g = two_player_game();
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g.add_card_to_battlefield(0, catalog::experimental_confectioner());
        let food = g.add_token_to_battlefield(0, &crabomination::game::effects::food_token());
        g.players[0].mana_pool.add_colorless(2);
        activate(&mut g, food, 0); // {2}, {T}, Sac: gain 3 life
        assert!(
            !g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Food"),
            "Food sacrificed",
        );
        assert!(
            g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Rat"),
            "sacrificing a Food made a Rat",
        );
    }

    /// Tangled Colony dies to N damage and leaves N Rats (CR 120 marked damage via
    /// leaves-battlefield LKI).
    #[test]
    fn tangled_colony_leaves_rats_equal_to_damage() {
        let mut g = two_player_game();
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        let colony = g.add_card_to_battlefield(0, catalog::tangled_colony()); // 3/2
        let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
        g.players[0].mana_pool.add(Color::Red, 1);
        cast(&mut g, bolt, Some(Target::Permanent(colony)), None); // 3 damage
        assert!(g.battlefield_find(colony).is_none(), "3 damage killed the 3/2");
        let rats = g
            .battlefield
            .iter()
            .filter(|c| c.controller == 0 && c.definition.name == "Rat")
            .count();
        assert_eq!(rats, 3, "one Rat per point of damage dealt this turn");
    }

    /// Specter of Mortality exiles creature cards from your graveyard to shrink
    /// every other creature by -X/-X.
    #[test]
    fn specter_of_mortality_team_debuff() {
        let mut g = two_player_game();
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
        let gy1 = g.add_card_to_graveyard(0, catalog::grizzly_bears());
        let gy2 = g.add_card_to_graveyard(0, catalog::grizzly_bears());
        let spectre = g.add_card_to_hand(0, catalog::specter_of_mortality());
        g.players[0].mana_pool.add(Color::Black, 2);
        g.players[0].mana_pool.add_colorless(3);
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Cards(vec![gy1, gy2])]));
        cast(&mut g, spectre, None, None);
        // -2/-2 turns the 2/2 into a 0/0 → dies; the Specter (3/3) is unaffected.
        assert!(g.battlefield_find(victim).is_none(), "other creature got -2/-2 and died");
        let cp = g.computed_permanent(spectre).unwrap();
        assert_eq!((cp.power, cp.toughness), (3, 3), "Specter itself unaffected (each *other*)");
    }

    /// Bargained Torch the Tower deals 3, scries, and exiles a lethally-damaged
    /// target instead of letting it die to the graveyard.
    #[test]
    fn torch_the_tower_bargained_exiles_target() {
        let mut g = two_player_game();
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        g.step = TurnStep::PreCombatMain;
        let target = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
        let fodder = g.add_token_to_battlefield(0, &crabomination::game::effects::food_token());
        let id = g.add_card_to_hand(0, catalog::torch_the_tower());
        g.players[0].mana_pool.add(Color::Red, 1);
        g.perform_action(GameAction::CastSpellBargain {
            card_id: id,
            sacrifice: Some(fodder),
            target: Some(Target::Permanent(target)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .expect("cast Torch the Tower bargained");
        drain_stack(&mut g);
        assert!(g.battlefield_find(target).is_none(), "3 damage killed the 2/2");
        assert!(
            g.exile.iter().any(|c| c.definition.name == "Grizzly Bears"),
            "lethally-damaged target exiled, not buried",
        );
        assert!(
            !g.players[1].graveyard.iter().any(|c| c.definition.name == "Grizzly Bears"),
            "not in the graveyard",
        );
    }
}

mod recent139 {
    use crabomination::catalog;
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    use crabomination::game::types::{Attack, AttackTarget, Target};
    use crabomination::game::*;
    use crabomination::game::{drain_stack, two_player_game};
    use crabomination::mana::Color;

    fn cast(g: &mut GameState, id: CardId, target: Option<Target>, x: Option<u32>) {
        g.perform_action(GameAction::CastSpell {
            card_id: id,
            target,
            additional_targets: vec![],
            mode: None,
            x_value: x,
        })
        .expect("cast");
        drain_stack(g);
    }

    fn rat_count(g: &GameState) -> usize {
        g.battlefield.iter().filter(|c| c.controller == 0 && c.definition.name == "Rat").count()
    }

    /// Misleading Motes sends a creature to its owner's library.
    #[test]
    fn misleading_motes_bounces_to_library() {
        let mut g = two_player_game();
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        let enemy = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let spell = g.add_card_to_hand(0, catalog::misleading_motes());
        g.players[0].mana_pool.add(Color::Blue, 1);
        g.players[0].mana_pool.add_colorless(3);
        cast(&mut g, spell, Some(Target::Permanent(enemy)), None);
        assert!(g.battlefield_find(enemy).is_none(), "creature left the battlefield");
        assert!(
            g.players[1].library.iter().any(|c| c.definition.name == "Grizzly Bears"),
            "put into its owner's library",
        );
    }

    /// Taken by Nightmares exiles a creature.
    #[test]
    fn taken_by_nightmares_exiles() {
        let mut g = two_player_game();
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        let enemy = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let spell = g.add_card_to_hand(0, catalog::taken_by_nightmares());
        g.players[0].mana_pool.add(Color::Black, 2);
        g.players[0].mana_pool.add_colorless(2);
        cast(&mut g, spell, Some(Target::Permanent(enemy)), None);
        assert!(g.battlefield_find(enemy).is_none(), "creature exiled");
        assert!(g.exile.iter().any(|c| c.definition.name == "Grizzly Bears"), "in exile");
    }

    /// Faerie Fencing gives -X/-X, plus -3/-3 more when you control a Faerie.
    #[test]
    fn faerie_fencing_faerie_bonus() {
        let mut g = two_player_game();
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        // A Faerie you control turns X=1 into a lethal -4/-4 on a 3/3.
        g.add_card_to_battlefield(0, catalog::spellstutter_sprite());
        let target = g.add_card_to_battlefield(1, catalog::centaur_courser()); // 3/3
        let spell = g.add_card_to_hand(0, catalog::faerie_fencing());
        g.players[0].mana_pool.add(Color::Black, 1);
        g.players[0].mana_pool.add_colorless(1); // X = 1
        cast(&mut g, spell, Some(Target::Permanent(target)), Some(1));
        assert!(g.battlefield_find(target).is_none(), "-1/-1 plus Faerie -3/-3 = -4/-4 killed the 3/3");
    }

    /// Flick a Coin pings, makes a Treasure, and draws.
    #[test]
    fn flick_a_coin_ping_treasure_draw() {
        let mut g = two_player_game();
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g.add_card_to_library(0, catalog::grizzly_bears());
        let spell = g.add_card_to_hand(0, catalog::flick_a_coin());
        let before = g.players[0].hand.len();
        g.players[0].mana_pool.add(Color::Red, 1);
        g.players[0].mana_pool.add_colorless(2);
        let life = g.players[1].life;
        cast(&mut g, spell, Some(Target::Player(1)), None);
        assert_eq!(g.players[1].life, life - 1, "1 damage to the player");
        assert!(g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Treasure"), "made a Treasure");
        assert_eq!(g.players[0].hand.len(), before, "-1 spell +1 draw = even");
    }

    /// Frantic Firebolt scales with instant/sorcery cards in your graveyard.
    #[test]
    fn frantic_firebolt_scales_with_graveyard() {
        let mut g = two_player_game();
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g.add_card_to_graveyard(0, catalog::lightning_bolt()); // instant
        g.add_card_to_graveyard(0, catalog::lightning_bolt()); // instant
        let target = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4
        let spell = g.add_card_to_hand(0, catalog::frantic_firebolt());
        g.players[0].mana_pool.add(Color::Red, 1);
        g.players[0].mana_pool.add_colorless(2);
        cast(&mut g, spell, Some(Target::Permanent(target)), None);
        // 2 + 2 instants = 4 damage → kills the 4/4.
        assert!(g.battlefield_find(target).is_none(), "4 damage killed the 4/4");
    }

    /// Ogre Chitterlord makes two Rats when it enters.
    #[test]
    fn ogre_chitterlord_makes_rats() {
        let mut g = two_player_game();
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        let id = g.add_card_to_hand(0, catalog::ogre_chitterlord());
        g.players[0].mana_pool.add(Color::Red, 2);
        g.players[0].mana_pool.add_colorless(4);
        cast(&mut g, id, None, None);
        assert_eq!(rat_count(&g), 2, "two Rats on entry");
    }

    /// Redcap Gutter-Dweller makes two Rats when it enters.
    #[test]
    fn redcap_gutter_dweller_makes_rats() {
        let mut g = two_player_game();
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        let id = g.add_card_to_hand(0, catalog::redcap_gutter_dweller());
        g.players[0].mana_pool.add(Color::Red, 2);
        g.players[0].mana_pool.add_colorless(2);
        cast(&mut g, id, None, None);
        assert_eq!(rat_count(&g), 2, "two Rats on entry");
    }

    /// Shatter the Oath destroys a creature and hangs a Wicked Role on your creature.
    #[test]
    fn shatter_the_oath_destroy_and_role() {
        let mut g = two_player_game();
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let enemy = g.add_card_to_battlefield(1, catalog::serra_angel());
        let spell = g.add_card_to_hand(0, catalog::shatter_the_oath());
        g.players[0].mana_pool.add(Color::Black, 2);
        g.players[0].mana_pool.add_colorless(3);
        cast(&mut g, spell, Some(Target::Permanent(enemy)), None);
        assert!(g.battlefield_find(enemy).is_none(), "target destroyed");
        assert!(
            g.battlefield.iter().any(|c| c.attached_to == Some(mine) && c.definition.name == "Wicked"),
            "Wicked Role attached to your creature",
        );
    }

    /// Tattered Ratter pumps a Rat that becomes blocked.
    #[test]
    fn tattered_ratter_pumps_blocked_rat() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::tattered_ratter());
        // A 1/1 Rat attacker.
        let attacker = g.add_card_to_battlefield(0, {
            use crabomination::card::{CardDefinition, CardType, CreatureType, Subtypes};
            CardDefinition {
                name: "Test Rat",
                power: 1,
                toughness: 1,
                card_types: vec![CardType::Creature],
                subtypes: Subtypes { creature_types: vec![CreatureType::Rat], ..Default::default() },
                ..Default::default()
            }
        });
        g.clear_sickness(attacker);
        let blocker = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        g.clear_sickness(blocker);
        g.step = TurnStep::DeclareAttackers;
        g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker,
            target: AttackTarget::Player(1),
        }]))
        .unwrap();
        g.step = TurnStep::DeclareBlockers;
        g.perform_action(GameAction::DeclareBlockers(vec![(blocker, attacker)])).unwrap();
        drain_stack(&mut g);
        let cp = g.computed_permanent(attacker).unwrap();
        assert_eq!((cp.power, cp.toughness), (3, 1), "blocked Rat got +2/+0");
    }

    /// Redtooth Vanguard returns from the graveyard when an enchantment enters.
    #[test]
    fn redtooth_vanguard_recurs_on_enchantment() {
        let mut g = two_player_game();
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g.add_card_to_graveyard(0, catalog::redtooth_vanguard());
        // Cast an enchantment; the graveyard trigger offers to pay {2}.
        let ench = g.add_card_to_hand(0, catalog::a_tale_for_the_ages());
        g.players[0].mana_pool.add(Color::White, 1);
        g.players[0].mana_pool.add_colorless(3); // {1} for the aura + {2} to recur
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
        cast(&mut g, ench, None, None);
        assert!(
            g.players[0].hand.iter().any(|c| c.definition.name == "Redtooth Vanguard"),
            "Redtooth Vanguard returned to hand",
        );
    }
}

mod recent140 {
    use crabomination::catalog;
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    use crabomination::game::types::Target;
    use crabomination::game::*;
    use crabomination::game::{drain_stack, two_player_game};
    use crabomination::mana::Color;

    fn cast(g: &mut GameState, id: CardId, target: Option<Target>, x: Option<u32>) {
        g.perform_action(GameAction::CastSpell {
            card_id: id,
            target,
            additional_targets: vec![],
            mode: None,
            x_value: x,
        })
        .expect("cast");
        drain_stack(g);
    }

    /// Food Coma exiles an opponent's creature and makes a Food; it returns when the
    /// enchantment leaves.
    #[test]
    fn food_coma_exiles_until_leaves() {
        let mut g = two_player_game();
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        let victim = g.add_card_to_battlefield(1, catalog::serra_angel());
        let coma = g.add_card_to_hand(0, catalog::food_coma());
        g.players[0].mana_pool.add(Color::White, 1);
        g.players[0].mana_pool.add_colorless(3);
        cast(&mut g, coma, Some(Target::Permanent(victim)), None);
        assert!(g.battlefield_find(victim).is_none(), "opponent creature exiled");
        assert!(g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Food"), "made a Food");
        // Destroy Food Coma → the creature returns.
        let coma_id = g.battlefield.iter().find(|c| c.definition.name == "Food Coma").unwrap().id;
        let ctx = crabomination::game::effects::EffectContext::for_ability(coma_id, 0, Some(Target::Permanent(coma_id)));
        g.resolve_effect(&crabomination::effect::Effect::Destroy { what: crabomination::effect::Selector::Target(0) }, &ctx)
            .unwrap();
        g.dispatch_triggers_for_events(&[]);
        drain_stack(&mut g);
        assert!(
            g.battlefield.iter().any(|c| c.definition.name == "Serra Angel"),
            "exiled creature returned when Food Coma left",
        );
    }

    /// Rankle's Prank mode "each player loses 4 life" hits both players.
    #[test]
    fn rankles_prank_loses_life_both() {
        let mut g = two_player_game();
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        let spell = g.add_card_to_hand(0, catalog::rankles_prank());
        g.players[0].mana_pool.add(Color::Black, 2);
        g.players[0].mana_pool.add_colorless(2);
        let (l0, l1) = (g.players[0].life, g.players[1].life);
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Modes(vec![1])]));
        cast(&mut g, spell, None, None);
        assert_eq!(g.players[0].life, l0 - 4, "you lose 4");
        assert_eq!(g.players[1].life, l1 - 4, "opponent loses 4");
    }

    /// Song of Totentanz makes X Rats.
    #[test]
    fn song_of_totentanz_makes_x_rats() {
        let mut g = two_player_game();
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        let spell = g.add_card_to_hand(0, catalog::song_of_totentanz());
        g.players[0].mana_pool.add(Color::Red, 1);
        g.players[0].mana_pool.add_colorless(3); // X = 3
        cast(&mut g, spell, None, Some(3));
        let rats = g.battlefield.iter().filter(|c| c.controller == 0 && c.definition.name == "Rat").count();
        assert_eq!(rats, 3, "X=3 → three Rats");
    }
}

mod recent141 {
    use crabomination::catalog;
    use crabomination::card::CounterType;
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    use crabomination::game::types::Target;
    use crabomination::game::*;
    use crabomination::game::{drain_stack, two_player_game};
    use crabomination::mana::Color;

    fn cast(g: &mut GameState, id: CardId, target: Option<Target>, x: Option<u32>) {
        g.perform_action(GameAction::CastSpell {
            card_id: id,
            target,
            additional_targets: vec![],
            mode: None,
            x_value: x,
        })
        .expect("cast");
        drain_stack(g);
    }

    /// Lady of Laughter draws at your end step when Celebration is active.
    #[test]
    fn lady_of_laughter_celebration_draw() {
        let mut g = two_player_game();
        g.active_player_idx = 0;
        g.add_card_to_battlefield(0, catalog::lady_of_laughter());
        g.add_card_to_library(0, catalog::grizzly_bears());
        let hand = g.players[0].hand.len();
        g.players[0].nonland_permanents_entered_this_turn = 2;
        g.fire_step_triggers(TurnStep::End);
        drain_stack(&mut g);
        assert_eq!(g.players[0].hand.len(), hand + 1, "Celebration end-step draw");
    }

    /// Sharae taps an opponent's creature and puts a stun counter on it.
    #[test]
    fn sharae_taps_and_stuns() {
        let mut g = two_player_game();
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        let enemy = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let id = g.add_card_to_hand(0, catalog::sharae_of_numbing_depths());
        g.players[0].mana_pool.add(Color::White, 1);
        g.players[0].mana_pool.add(Color::Blue, 1);
        g.players[0].mana_pool.add_colorless(2);
        cast(&mut g, id, Some(Target::Permanent(enemy)), None);
        let e = g.battlefield_find(enemy).unwrap();
        assert!(e.tapped, "opponent creature tapped");
        assert_eq!(e.counter_count(CounterType::Stun), 1, "stun counter added");
    }

    /// Sharae's "whenever you tap …" fires only when the tap is your effect, not
    /// when the opponent's own creature becomes tapped.
    #[test]
    fn sharae_you_tapped_actor_gating() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::sharae_of_numbing_depths());
        let enemy = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        g.add_card_to_library(0, catalog::grizzly_bears());
        g.add_card_to_library(0, catalog::grizzly_bears());
        // Opponent taps their own creature — no draw.
        let hand = g.players[0].hand.len();
        g.dispatch_triggers_for_events(&[GameEvent::PermanentTapped { card_id: enemy, actor: Some(1), as_attacker: false }]);
        drain_stack(&mut g);
        assert_eq!(g.players[0].hand.len(), hand, "opponent self-tap does not trigger");
        // You tap it — draw once.
        g.dispatch_triggers_for_events(&[GameEvent::PermanentTapped { card_id: enemy, actor: Some(0), as_attacker: false }]);
        drain_stack(&mut g);
        assert_eq!(g.players[0].hand.len(), hand + 1, "your tap draws a card");
    }

    /// Ingenious Prodigy enters with X +1/+1 counters and cashes one for a card.
    #[test]
    fn ingenious_prodigy_x_counters_then_draw() {
        let mut g = two_player_game();
        g.active_player_idx = 0;
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        let id = g.add_card_to_hand(0, catalog::ingenious_prodigy());
        g.players[0].mana_pool.add(Color::Blue, 1);
        g.players[0].mana_pool.add_colorless(2); // X = 2
        g.add_card_to_library(0, catalog::grizzly_bears());
        cast(&mut g, id, None, Some(2));
        let prod = g.battlefield.iter().find(|c| c.definition.name == "Ingenious Prodigy").unwrap().id;
        assert_eq!(g.battlefield_find(prod).unwrap().counter_count(CounterType::PlusOnePlusOne), 2);
        let hand = g.players[0].hand.len();
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
        g.fire_step_triggers(TurnStep::Upkeep);
        drain_stack(&mut g);
        assert_eq!(g.battlefield_find(prod).unwrap().counter_count(CounterType::PlusOnePlusOne), 1, "spent a counter");
        assert_eq!(g.players[0].hand.len(), hand + 1, "drew a card");
    }

    /// Malevolent Witchkite sacrifices tokens on entry and draws that many.
    #[test]
    fn malevolent_witchkite_sacs_and_draws() {
        let mut g = two_player_game();
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g.add_token_to_battlefield(0, &crabomination::game::effects::food_token());
        g.add_token_to_battlefield(0, &crabomination::game::effects::treasure_token());
        let id = g.add_card_to_hand(0, catalog::malevolent_witchkite());
        g.add_card_to_library(0, catalog::grizzly_bears());
        g.add_card_to_library(0, catalog::grizzly_bears());
        g.players[0].mana_pool.add(Color::Black, 2);
        g.players[0].mana_pool.add_colorless(4);
        let hand = g.players[0].hand.len();
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Amount(2)]));
        cast(&mut g, id, None, None);
        // Witchkite leaves hand (−1) and two draws (+2) → net +1.
        assert_eq!(hand + 1, g.players[0].hand.len(), "drew two for two sacrifices");
        assert!(!g.battlefield.iter().any(|c| c.is_token), "both tokens sacrificed");
    }

    /// Obyra drains each opponent when another Faerie you control enters.
    #[test]
    fn obyra_drains_on_faerie_entry() {
        let mut g = two_player_game();
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g.add_card_to_battlefield(0, catalog::obyra_dreaming_duelist());
        let faerie = g.add_card_to_hand(0, catalog::spellstutter_sprite());
        g.players[0].mana_pool.add(Color::Blue, 1);
        g.players[0].mana_pool.add_colorless(1);
        let life = g.players[1].life;
        cast(&mut g, faerie, None, None);
        assert_eq!(g.players[1].life, life - 1, "opponent loses 1 on Faerie ETB");
    }

    /// Old Flitterfang makes a Food at end step when a creature died this turn.
    #[test]
    fn old_flitterfang_end_step_food() {
        let mut g = two_player_game();
        g.active_player_idx = 0;
        g.add_card_to_battlefield(0, catalog::old_flitterfang());
        g.players[0].creatures_died_this_turn = 1;
        g.fire_step_triggers(TurnStep::End);
        drain_stack(&mut g);
        assert!(g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Food"), "made a Food");
    }

    /// Unruly Catapult untaps when you cast an instant.
    #[test]
    fn unruly_catapult_untaps_on_instant() {
        let mut g = two_player_game();
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        let cat = g.add_card_to_battlefield(0, catalog::unruly_catapult());
        g.battlefield_find_mut(cat).unwrap().tapped = true;
        let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
        g.players[0].mana_pool.add(Color::Red, 1);
        cast(&mut g, bolt, Some(Target::Player(1)), None);
        assert!(!g.battlefield_find(cat).unwrap().tapped, "catapult untapped on instant cast");
    }

    /// Realm-Scorcher Hellkite, when bargained, adds four mana on entry.
    #[test]
    fn realm_scorcher_bargained_ramps() {
        let mut g = two_player_game();
        g.active_player_idx = 0;
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        let token = g.add_token_to_battlefield(0, &crabomination::game::effects::treasure_token());
        let id = g.add_card_to_hand(0, catalog::realm_scorcher_hellkite());
        g.players[0].mana_pool.add(Color::Red, 2);
        g.players[0].mana_pool.add_colorless(4);
        g.perform_action(GameAction::CastSpellBargain {
            card_id: id,
            sacrifice: Some(token),
            target: None,
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .unwrap();
        drain_stack(&mut g);
        assert_eq!(g.players[0].mana_pool.total(), 4, "bargained ETB floated four mana");
    }

    /// Tough Cookie mints a Food and can animate a noncreature artifact into a 4/4.
    #[test]
    fn tough_cookie_food_and_animate() {
        let mut g = two_player_game();
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        let id = g.add_card_to_hand(0, catalog::tough_cookie());
        g.players[0].mana_pool.add(Color::Green, 1);
        g.players[0].mana_pool.add_colorless(1);
        cast(&mut g, id, None, None);
        let cookie = g.battlefield.iter().find(|c| c.definition.name == "Tough Cookie").unwrap().id;
        let food = g.battlefield.iter().find(|c| c.definition.name == "Food").unwrap().id;
        g.players[0].mana_pool.add(Color::Green, 1);
        g.players[0].mana_pool.add_colorless(2);
        g.perform_action(GameAction::ActivateAbility {
            card_id: cookie,
            ability_index: 0,
            target: Some(Target::Permanent(food)),
            additional_targets: vec![],
            x_value: None, mode: None,
        })
        .unwrap();
        drain_stack(&mut g);
        let f = g.computed_permanent(food).unwrap();
        assert_eq!((f.power, f.toughness), (4, 4), "Food animated to 4/4");
        assert!(f.card_types.contains(&crabomination::card::CardType::Creature), "now a creature");
    }
}

mod recent142 {
    use crabomination::catalog;
    use crabomination::card::CounterType;
    use crabomination::game::types::Target;
    use crabomination::game::*;
    use crabomination::game::{drain_stack, two_player_game};
    use crabomination::mana::Color;

    fn cast(g: &mut GameState, id: CardId, target: Option<Target>, x: Option<u32>) {
        g.perform_action(GameAction::CastSpell {
            card_id: id,
            target,
            additional_targets: vec![],
            mode: None,
            x_value: x,
        })
        .expect("cast");
        drain_stack(g);
    }

    /// Greta enters with a Food and can sacrifice it to grow a creature.
    #[test]
    fn greta_food_and_counter() {
        let mut g = two_player_game();
        g.active_player_idx = 0;
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        let id = g.add_card_to_hand(0, catalog::greta_sweettooth_scourge());
        g.players[0].mana_pool.add(Color::Black, 1);
        g.players[0].mana_pool.add(Color::Green, 1);
        g.players[0].mana_pool.add_colorless(1);
        cast(&mut g, id, None, None);
        let greta = g.battlefield.iter().find(|c| c.definition.name.starts_with("Greta")).unwrap().id;
        assert!(g.battlefield.iter().any(|c| c.definition.name == "Food"), "made a Food");
        g.players[0].mana_pool.add(Color::Green, 1);
        g.perform_action(GameAction::ActivateAbility {
            card_id: greta,
            ability_index: 0,
            target: Some(Target::Permanent(greta)),
            additional_targets: vec![],
            x_value: None, mode: None,
        })
        .expect("sac a Food for a counter");
        drain_stack(&mut g);
        assert_eq!(g.battlefield_find(greta).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
        assert!(!g.battlefield.iter().any(|c| c.definition.name == "Food"), "Food sacrificed");
    }

    /// Totentanz makes a Rat when a nontoken creature you control dies.
    #[test]
    fn totentanz_rat_on_death() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::totentanz_swarm_piper());
        let victim = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.battlefield_find_mut(victim).unwrap().damage = 2; // lethal → CreatureDied
        let evs = g.check_state_based_actions();
        g.dispatch_triggers_for_events(&evs);
        drain_stack(&mut g);
        assert!(g.battlefield.iter().any(|c| c.definition.name == "Rat"), "Rat minted on death");
    }

    /// Neva returns an enchantment card from your graveyard on entry, and grows +
    /// scries when an enchantment you control dies.
    #[test]
    fn neva_returns_and_grows() {
        let mut g = two_player_game();
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        let ench = g.add_card_to_graveyard(0, catalog::a_tale_for_the_ages());
        let id = g.add_card_to_hand(0, catalog::neva_stalked_by_nightmares());
        g.players[0].mana_pool.add(Color::White, 1);
        g.players[0].mana_pool.add(Color::Black, 1);
        g.players[0].mana_pool.add_colorless(2);
        g.add_card_to_library(0, catalog::grizzly_bears()); // for the scry
        cast(&mut g, id, Some(Target::Permanent(ench)), None);
        assert!(g.players[0].hand.iter().any(|c| c.id == ench), "enchantment returned to hand");
        let neva = g.battlefield.iter().find(|c| c.definition.name.starts_with("Neva")).unwrap().id;
        let onbf = g.add_card_to_battlefield(0, catalog::a_tale_for_the_ages());
        let ctx = crabomination::game::effects::EffectContext::for_ability(onbf, 0, None);
        let evs = g.resolve_effect(
            &crabomination::effect::Effect::Destroy { what: crabomination::effect::Selector::This },
            &ctx,
        )
        .unwrap();
        g.dispatch_triggers_for_events(&evs);
        drain_stack(&mut g);
        assert_eq!(g.battlefield_find(neva).unwrap().counter_count(CounterType::PlusOnePlusOne), 1, "Neva grew");
    }

    /// Syr Armont hangs a Monster Role and his anthem stacks: a 2/2 becomes 4/4.
    #[test]
    fn syr_armont_role_and_anthem() {
        let mut g = two_player_game();
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let id = g.add_card_to_hand(0, catalog::syr_armont_the_redeemer());
        g.players[0].mana_pool.add(Color::Green, 1);
        g.players[0].mana_pool.add(Color::White, 1);
        g.players[0].mana_pool.add_colorless(3);
        cast(&mut g, id, Some(Target::Permanent(bear)), None);
        let b = g.computed_permanent(bear).unwrap();
        assert_eq!((b.power, b.toughness), (4, 4), "+1/+1 from Role and +1/+1 from anthem");
    }

    /// Troyan's restricted mana funds an expensive spell but not a cheap one.
    #[test]
    fn troyan_high_mv_or_x_restriction() {
        use crabomination::mana::{SpellKind, SpendRestriction};
        let r = SpendRestriction::HighMvOrX;
        let big = SpellKind { mana_value: 6, ..Default::default() };
        let xspell = SpellKind { mana_value: 1, has_x: true, ..Default::default() };
        let cheap = SpellKind { mana_value: 3, ..Default::default() };
        assert!(r.allows(&big), "funds a mana value 5+ spell");
        assert!(r.allows(&xspell), "funds an {{X}} spell");
        assert!(!r.allows(&cheap), "not a small non-X spell");
        // The ability floats restricted mana into the pool.
        let mut g = two_player_game();
        let troyan = g.add_card_to_battlefield(0, catalog::troyan_gutsy_explorer());
        g.battlefield_find_mut(troyan).unwrap().summoning_sick = false;
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::ActivateAbility {
            card_id: troyan,
            ability_index: 0,
            target: None,
            additional_targets: vec![],
            x_value: None, mode: None,
        })
        .expect("tap for restricted mana");
        assert_eq!(g.players[0].mana_pool.restricted_total(), 2, "two restricted mana floated");
    }

    /// Johann lets you cast one instant/sorcery from the top of your library each
    /// turn; the second top card can't be cast until the cap resets.
    #[test]
    fn johann_casts_from_top_once_per_turn() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::johann_apprentice_sorcerer());
        let bolt1 = g.add_card_to_library(0, catalog::lightning_bolt()); // top of library
        let bolt2 = g.add_card_to_library(0, catalog::lightning_bolt());
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g.players[0].mana_pool.add(Color::Red, 2);
        cast(&mut g, bolt1, Some(Target::Player(1)), None);
        assert_eq!(g.players[1].life, 17, "first Bolt cast from top dealt 3");
        assert!(
            g.perform_action(GameAction::CastSpell {
                card_id: bolt2,
                target: Some(Target::Player(1)),
                additional_targets: vec![],
                mode: None,
                x_value: None,
            })
            .is_err(),
            "second top-of-library cast is blocked by the once-per-turn cap",
        );
        assert_eq!(g.players[1].life, 17, "no second Bolt this turn");
    }

    /// Solitary Sanctuary grows one of your creatures when you tap an enemy creature.
    #[test]
    fn solitary_sanctuary_tap_grows() {
        let mut g = two_player_game();
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let enemy = g.add_card_to_battlefield(1, catalog::serra_angel());
        let id = g.add_card_to_hand(0, catalog::solitary_sanctuary());
        g.players[0].mana_pool.add(Color::White, 1);
        g.players[0].mana_pool.add_colorless(2);
        cast(&mut g, id, Some(Target::Permanent(enemy)), None);
        assert!(g.battlefield_find(enemy).unwrap().tapped, "enemy tapped by ETB");
        // The ETB tap is itself a "you tap …" event, so your creature grows.
        assert_eq!(g.battlefield_find(mine).unwrap().counter_count(CounterType::PlusOnePlusOne), 1, "your creature grew");
    }

    /// Farsight Ritual digs four and pulls two into hand.
    #[test]
    fn farsight_ritual_digs_two() {
        let mut g = two_player_game();
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        for _ in 0..4 { g.add_card_to_library(0, catalog::grizzly_bears()); }
        let id = g.add_card_to_hand(0, catalog::farsight_ritual());
        g.players[0].mana_pool.add(Color::Blue, 2);
        g.players[0].mana_pool.add_colorless(2);
        let hand = g.players[0].hand.len();
        cast(&mut g, id, None, None);
        // Ritual leaves hand (−1), two cards drawn (+2) → net +1.
        assert_eq!(g.players[0].hand.len(), hand + 1, "two cards taken to hand");
    }
}

mod recent143 {
    use crabomination::catalog;
    use crabomination::game::types::Target;
    use crabomination::game::*;
    use crabomination::game::{drain_stack, two_player_game};
    use crabomination::mana::Color;

    fn cast(g: &mut GameState, id: CardId, target: Option<Target>) {
        g.perform_action(GameAction::CastSpell {
            card_id: id,
            target,
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .expect("cast");
        drain_stack(g);
    }

    fn cast_adventure(g: &mut GameState, id: CardId, target: Option<Target>) {
        g.perform_action(GameAction::CastAdventure {
            card_id: id,
            target,
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .expect("cast adventure");
        drain_stack(g);
    }

    /// Ferocious Werefox's Guard Change adventure hangs a Monster Role (+1/+1 trample).
    #[test]
    fn guard_change_hangs_monster_role() {
        let mut g = two_player_game();
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let id = g.add_card_to_hand(0, catalog::ferocious_werefox());
        g.players[0].mana_pool.add(Color::Green, 1);
        g.players[0].mana_pool.add_colorless(1);
        cast_adventure(&mut g, id, Some(Target::Permanent(bear)));
        let b = g.computed_permanent(bear).unwrap();
        assert_eq!(b.power, 3, "2/2 + Monster Role +1/+1");
        assert!(b.keywords.contains(&crabomination::card::Keyword::Trample), "Role grants trample");
    }

    /// Hare Raising pumps a creature by the number of creatures you control.
    #[test]
    fn hare_raising_scales_with_board() {
        let mut g = two_player_game();
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        let a = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 3 creatures
        let id = g.add_card_to_hand(0, catalog::pollen_shield_hare());
        g.players[0].mana_pool.add(Color::Green, 1);
        cast_adventure(&mut g, id, Some(Target::Permanent(a)));
        let c = g.computed_permanent(a).unwrap();
        assert_eq!(c.power, 5, "2 + 3 creatures");
        assert!(c.keywords.contains(&crabomination::card::Keyword::Vigilance), "gains vigilance");
    }

    /// Frolicking Familiar grows when you cast an instant.
    #[test]
    fn frolicking_familiar_grows_on_instant() {
        let mut g = two_player_game();
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        let fam = g.add_card_to_battlefield(0, catalog::frolicking_familiar());
        let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
        g.players[0].mana_pool.add(Color::Red, 1);
        cast(&mut g, bolt, Some(Target::Player(1)));
        assert_eq!(g.computed_permanent(fam).unwrap().power, 3, "+1/+1 on instant cast");
    }

    /// Gumdrop Poisoner's ETB shrinks a creature by the life you gained this turn.
    #[test]
    fn gumdrop_poisoner_minus_x() {
        let mut g = two_player_game();
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g.players[0].life_gained_this_turn = 3;
        let victim = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4
        let id = g.add_card_to_hand(0, catalog::gumdrop_poisoner());
        g.players[0].mana_pool.add(Color::Black, 1);
        g.players[0].mana_pool.add_colorless(2);
        cast(&mut g, id, Some(Target::Permanent(victim)));
        assert_eq!(g.computed_permanent(victim).unwrap().power, 1, "-3/-3 from 3 life gained");
    }

    /// Croaking Curse taps a creature and makes it 1/1 with a Cursed Role.
    #[test]
    fn croaking_curse_taps_and_shrinks() {
        let mut g = two_player_game();
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        let enemy = g.add_card_to_battlefield(1, catalog::serra_angel());
        let id = g.add_card_to_hand(0, catalog::vantress_transmuter());
        g.players[0].mana_pool.add(Color::Blue, 1);
        g.players[0].mana_pool.add_colorless(1);
        cast_adventure(&mut g, id, Some(Target::Permanent(enemy)));
        assert!(g.battlefield_find(enemy).unwrap().tapped, "tapped");
        assert_eq!(g.computed_permanent(enemy).unwrap().power, 1, "Cursed Role sets it to 1/1");
    }

    /// Free the Fae mills four and takes an instant/sorcery/Faerie to hand.
    #[test]
    fn free_the_fae_mills_and_takes() {
        let mut g = two_player_game();
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        let bolt = g.add_card_to_library(0, catalog::lightning_bolt()); // an instant among the top four
        for _ in 0..3 { g.add_card_to_library(0, catalog::grizzly_bears()); }
        let id = g.add_card_to_hand(0, catalog::picklock_prankster());
        g.players[0].mana_pool.add(Color::Blue, 1);
        g.players[0].mana_pool.add_colorless(1);
        g.decider = Box::new(crabomination::decision::ScriptedDecider::new([
            crabomination::decision::DecisionAnswer::Cards(vec![bolt]),
        ]));
        cast_adventure(&mut g, id, None);
        assert!(g.players[0].hand.iter().any(|c| c.definition.name == "Lightning Bolt"), "took the instant");
    }

    /// Bear Down destroys an artifact or enchantment.
    #[test]
    fn bear_down_destroys_enchantment() {
        let mut g = two_player_game();
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        let ench = g.add_card_to_battlefield(1, catalog::a_tale_for_the_ages());
        let id = g.add_card_to_hand(0, catalog::stormkeld_vanguard());
        g.players[0].mana_pool.add(Color::Green, 1);
        g.players[0].mana_pool.add_colorless(1);
        cast_adventure(&mut g, id, Some(Target::Permanent(ench)));
        assert!(g.battlefield_find(ench).is_none(), "enchantment destroyed");
    }

    /// Scalding Viper pings an opponent who casts a cheap spell.
    #[test]
    fn scalding_viper_pings_cheap_caster() {
        let mut g = two_player_game();
        g.step = TurnStep::PreCombatMain;
        g.add_card_to_battlefield(0, catalog::scalding_viper());
        let bolt = g.add_card_to_hand(1, catalog::lightning_bolt()); // MV 1
        g.players[1].mana_pool.add(Color::Red, 1);
        g.priority.player_with_priority = 1;
        let life = g.players[1].life;
        cast(&mut g, bolt, Some(Target::Player(0)));
        assert_eq!(g.players[1].life, life - 1, "opponent pinged for casting a MV≤3 spell");
    }
}

mod recent144 {
    use crabomination::catalog;
    use crabomination::card::CounterType;
    use crabomination::game::types::Target;
    use crabomination::game::*;
    use crabomination::game::{drain_stack, two_player_game};
    use crabomination::mana::Color;

    fn cast(g: &mut GameState, id: CardId, target: Option<Target>) {
        g.perform_action(GameAction::CastSpell {
            card_id: id,
            target,
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .expect("cast");
        drain_stack(g);
    }

    fn cast_adventure(g: &mut GameState, id: CardId, target: Option<Target>) {
        g.perform_action(GameAction::CastAdventure {
            card_id: id,
            target,
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .expect("cast adventure");
        drain_stack(g);
    }

    /// Icewrought Sentry grows when you tap an opponent's creature.
    #[test]
    fn icewrought_sentry_pumps_on_you_tap() {
        let mut g = two_player_game();
        let sentry = g.add_card_to_battlefield(0, catalog::icewrought_sentry());
        let enemy = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        g.dispatch_triggers_for_events(&[GameEvent::PermanentTapped { card_id: enemy, actor: Some(0), as_attacker: false }]);
        drain_stack(&mut g);
        let s = g.computed_permanent(sentry).unwrap();
        assert_eq!((s.power, s.toughness), (4, 4), "+2/+1 when you tap an enemy creature");
    }

    /// Galvanic Giant taps and stuns an opponent's creature when you cast a MV-5+ spell.
    #[test]
    fn galvanic_giant_high_mv_tap_stun() {
        let mut g = two_player_game();
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g.add_card_to_battlefield(0, catalog::galvanic_giant());
        let enemy = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let big = g.add_card_to_hand(0, catalog::serra_angel()); // MV 5
        g.players[0].mana_pool.add(Color::White, 2);
        g.players[0].mana_pool.add_colorless(3);
        cast(&mut g, big, None);
        let e = g.battlefield_find(enemy).unwrap();
        assert!(e.tapped, "enemy tapped");
        assert_eq!(e.counter_count(CounterType::Stun), 1, "stun counter added");
    }

    /// Aquatic Alchemist grows only on the first instant/sorcery each turn.
    #[test]
    fn aquatic_alchemist_first_spell_only() {
        let mut g = two_player_game();
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        let alch = g.add_card_to_battlefield(0, catalog::aquatic_alchemist());
        let b1 = g.add_card_to_hand(0, catalog::lightning_bolt());
        let b2 = g.add_card_to_hand(0, catalog::lightning_bolt());
        g.players[0].mana_pool.add(Color::Red, 2);
        cast(&mut g, b1, Some(Target::Player(1)));
        cast(&mut g, b2, Some(Target::Player(1)));
        assert_eq!(g.computed_permanent(alch).unwrap().power, 3, "+2/+0 once (first spell only)");
    }

    /// Rip the Seams destroys a tapped creature.
    #[test]
    fn rip_the_seams_destroys_tapped() {
        let mut g = two_player_game();
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        let enemy = g.add_card_to_battlefield(1, catalog::serra_angel());
        g.battlefield_find_mut(enemy).unwrap().tapped = true;
        let id = g.add_card_to_hand(0, catalog::threadbind_clique());
        g.players[0].mana_pool.add(Color::White, 1);
        g.players[0].mana_pool.add_colorless(2);
        cast_adventure(&mut g, id, Some(Target::Permanent(enemy)));
        assert!(g.battlefield_find(enemy).is_none(), "tapped creature destroyed");
    }

    /// Swift Spiral flickers a nontoken creature (exiled now, returns later).
    #[test]
    fn swift_spiral_flickers() {
        let mut g = two_player_game();
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let id = g.add_card_to_hand(0, catalog::twining_twins());
        g.players[0].mana_pool.add(Color::White, 1);
        g.players[0].mana_pool.add_colorless(1);
        cast_adventure(&mut g, id, Some(Target::Permanent(mine)));
        assert!(g.battlefield_find(mine).is_none(), "creature exiled by the flicker");
    }

    /// Spellscorn Coven makes each opponent discard on entry.
    #[test]
    fn spellscorn_coven_etb_discard() {
        let mut g = two_player_game();
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g.add_card_to_hand(1, catalog::grizzly_bears());
        let id = g.add_card_to_hand(0, catalog::spellscorn_coven());
        g.players[0].mana_pool.add(Color::Black, 1);
        g.players[0].mana_pool.add_colorless(3);
        let opp_hand = g.players[1].hand.len();
        cast(&mut g, id, None);
        assert_eq!(g.players[1].hand.len(), opp_hand - 1, "opponent discarded a card");
    }
}

mod recent145 {
    use crabomination::catalog;
    use crabomination::card::CounterType;
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    use crabomination::game::*;
    use crabomination::game::{drain_stack, two_player_game};

    /// Hylda's reflexive modal fires when you tap an opponent's creature and pay {1}.
    #[test]
    fn hylda_reflexive_modal_on_you_tap() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::hylda_of_the_icy_crown());
        let enemy = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        g.players[0].mana_pool.add_colorless(1); // to pay the reflexive {1}
        // Mode 0 = create a 4/4 Elemental.
        g.decider = Box::new(ScriptedDecider::new([
            DecisionAnswer::Bool(true),
            DecisionAnswer::Modes(vec![0]),
        ]));
        g.dispatch_triggers_for_events(&[GameEvent::PermanentTapped { card_id: enemy, actor: Some(0), as_attacker: false }]);
        drain_stack(&mut g);
        assert!(
            g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Elemental"),
            "paid {{1}} and made a 4/4 Elemental",
        );
    }

    /// Ash grows when attacking while Celebration is active.
    #[test]
    fn ash_celebration_attack_counter() {
        let mut g = two_player_game();
        g.active_player_idx = 0;
        let ash = g.add_card_to_battlefield(0, catalog::ash_party_crasher());
        g.clear_sickness(ash);
        g.players[0].nonland_permanents_entered_this_turn = 2;
        g.step = TurnStep::DeclareAttackers;
        g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker: ash,
            target: AttackTarget::Player(1),
        }]))
        .unwrap();
        drain_stack(&mut g);
        assert_eq!(g.battlefield_find(ash).unwrap().counter_count(CounterType::PlusOnePlusOne), 1, "Celebration grew Ash");
    }
}

mod recent146 {
    use crabomination::card::{CounterType, Keyword};
    use crabomination::catalog;
    use crabomination::game::*;
    use crabomination::game::two_player_game;
    use crabomination::mana::Color;

    /// Resolve a standalone effect controlled by `player` (test helper).
    fn resolve_for(g: &mut GameState, player: usize, effect: crabomination::effect::Effect) {
        let src = g.add_card_to_battlefield(player, catalog::grizzly_bears());
        let ctx = crabomination::game::effects::EffectContext::for_ability(src, player, None);
        let events = g.resolve_effect(&effect, &ctx).unwrap();
        g.dispatch_triggers_for_events(&events);
        drain_stack(g);
    }

    /// Bitter Chill's enchanted creature doesn't untap during its controller's
    /// untap step.
    #[test]
    fn bitter_chill_locks_untap() {
        let mut g = two_player_game();
        let chill = g.add_card_to_battlefield(0, catalog::bitter_chill());
        let creature = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        g.battlefield_find_mut(chill).unwrap().attached_to = Some(creature);
        g.battlefield_find_mut(creature).unwrap().tapped = true;
        g.active_player_idx = 1;
        g.do_untap();
        assert!(g.battlefield_find(creature).unwrap().tapped, "locked creature stays tapped");
    }

    /// Syr Ginger gains trample/hexproof/haste only while an opponent controls a
    /// planeswalker.
    #[test]
    fn syr_ginger_planeswalker_keywords() {
        let mut g = two_player_game();
        let ginger = g.add_card_to_battlefield(0, catalog::syr_ginger_the_meal_ender());
        assert!(
            !g.computed_permanent(ginger).unwrap().keywords.contains(&Keyword::Trample),
            "no keywords without an opposing planeswalker",
        );
        g.add_card_to_battlefield(1, catalog::karn_scion_of_urza());
        let kws = g.computed_permanent(ginger).unwrap().keywords.clone();
        assert!(kws.contains(&Keyword::Trample), "trample once opponent has a planeswalker");
        assert!(kws.contains(&Keyword::Hexproof), "hexproof too");
        assert!(kws.contains(&Keyword::Haste), "haste too");
    }

    /// A friendly artifact dying grows Syr Ginger; a friendly creature dying does
    /// not (the "another artifact" filter).
    #[test]
    fn syr_ginger_artifact_death_counter() {
        let mut g = two_player_game();
        let ginger = g.add_card_to_battlefield(0, catalog::syr_ginger_the_meal_ender());
        g.add_card_to_battlefield(0, catalog::mind_stone());
        // Destroy only the noncreature artifact (Syr Ginger is an artifact creature).
        resolve_for(&mut g, 0, crabomination::effect::Effect::Destroy {
            what: crabomination::effect::Selector::EachPermanent(
                crabomination::card::SelectionRequirement::Artifact
                    .and(crabomination::card::SelectionRequirement::Noncreature),
            ),
        });
        assert_eq!(
            g.battlefield_find(ginger).unwrap().counter_count(CounterType::PlusOnePlusOne),
            1,
            "an artifact dying added a +1/+1 counter",
        );
    }

    /// Archon of the Wild Rose sets your other enchanted creatures to base 4/4 with
    /// flying; an unenchanted creature is untouched.
    #[test]
    fn archon_buffs_enchanted_creatures() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::archon_of_the_wild_rose());
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let plain = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        // Enchant only `bear` with an Aura.
        let aura = g.add_card_to_battlefield(0, catalog::pacifism());
        g.battlefield_find_mut(aura).unwrap().attached_to = Some(bear);
        let buffed = g.computed_permanent(bear).unwrap();
        assert_eq!((buffed.power, buffed.toughness), (4, 4), "enchanted creature is base 4/4");
        assert!(buffed.keywords.contains(&Keyword::Flying), "and has flying");
        let plain_c = g.computed_permanent(plain).unwrap();
        assert_eq!((plain_c.power, plain_c.toughness), (2, 2), "unenchanted creature unchanged");
    }

    /// Back for Seconds returns two creatures to hand when not bargained.
    #[test]
    fn back_for_seconds_unbargained_returns_two() {
        let mut g = two_player_game();
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        for _ in 0..2 {
            g.add_card_to_graveyard(0, catalog::grizzly_bears());
        }
        let spell = g.add_card_to_hand(0, catalog::back_for_seconds());
        g.players[0].mana_pool.add(Color::Black, 1);
        g.players[0].mana_pool.add_colorless(2);
        g.perform_action(GameAction::CastSpell {
            card_id: spell,
            target: None,
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .expect("cast Back for Seconds");
        drain_stack(&mut g);
        let creatures_in_hand =
            g.players[0].hand.iter().filter(|c| c.definition.name == "Grizzly Bears").count();
        assert_eq!(creatures_in_hand, 2, "both creatures returned to hand");
    }

    /// Faunsbane Troll saccs its Monster Role to fight; the fought creature is
    /// exiled instead of dying.
    #[test]
    fn faunsbane_troll_fight_exiles() {
        let mut g = two_player_game();
        // The ETB mints a Monster Role attached to the Troll.
        let troll = g.move_card_to_battlefield_for_test(0, catalog::faunsbane_troll());
        drain_stack(&mut g);
        g.clear_sickness(troll);
        let role = g
            .battlefield
            .iter()
            .find(|c| c.definition.name == "Monster" && c.attached_to == Some(troll))
            .expect("Monster Role attached")
            .id;
        let prey = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g.players[0].mana_pool.add_colorless(1);
        g.perform_action(GameAction::ActivateAbility {
            card_id: troll,
            ability_index: 0,
            target: Some(Target::Permanent(prey)),
            additional_targets: vec![],
            x_value: None, mode: None,
        })
        .expect("activate Faunsbane fight");
        drain_stack(&mut g);
        assert!(g.battlefield_find(role).is_none(), "Monster Role sacrificed as a cost");
        assert!(g.exile.iter().any(|c| c.id == prey), "fought creature exiled, not in graveyard");
        assert!(g.battlefield_find(troll).is_some(), "Troll survives the 2-power back-swing");
    }

    /// Horned Loch-Whale enters tapped on an opponent's turn, untapped on yours.
    #[test]
    fn horned_loch_whale_conditional_enters_tapped() {
        let mut g = two_player_game();
        g.active_player_idx = 1; // opponent's turn
        let whale = g.move_card_to_battlefield_for_test(0, catalog::horned_loch_whale());
        assert!(g.battlefield_find(whale).unwrap().tapped, "enters tapped on opponent's turn");

        g.active_player_idx = 0; // your turn
        let whale2 = g.move_card_to_battlefield_for_test(0, catalog::horned_loch_whale());
        assert!(!g.battlefield_find(whale2).unwrap().tapped, "enters untapped on your turn");
    }
}

mod recent147 {
    use crabomination::card::CounterType;
    use crabomination::catalog;
    use crabomination::game::*;
    use crabomination::game::two_player_game;

    fn advance_to(g: &mut GameState, step: crabomination::game::TurnStep) {
        while g.step != step {
            g.perform_action(GameAction::PassPriority).expect("pass priority");
        }
    }

    /// Gingerbread Cabin enters tapped with fewer than three other Forests; with
    /// three it enters untapped and mints a Food.
    #[test]
    fn gingerbread_cabin_conditional_tap_and_food() {
        let mut g = two_player_game();
        let tapped = g.move_card_to_battlefield_for_test(0, catalog::gingerbread_cabin());
        drain_stack(&mut g);
        assert!(g.battlefield_find(tapped).unwrap().tapped, "enters tapped with no Forests");
        assert!(
            !g.battlefield.iter().any(|c| c.definition.name == "Food"),
            "no Food when it enters tapped",
        );
        for _ in 0..3 {
            g.add_card_to_battlefield(0, catalog::forest());
        }
        let untapped = g.move_card_to_battlefield_for_test(0, catalog::gingerbread_cabin());
        drain_stack(&mut g);
        assert!(!g.battlefield_find(untapped).unwrap().tapped, "untapped with three Forests");
        assert!(g.battlefield.iter().any(|c| c.definition.name == "Food"), "Food minted");
    }

    /// The Witch's Vanity chapter I destroys a small opposing creature.
    #[test]
    fn witchs_vanity_chapter_one_destroys_small_creature() {
        let mut g = two_player_game();
        let saga = g.add_card_to_battlefield(0, catalog::the_witchs_vanity());
        let small = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // MV 2
        g.saga_advance(saga);
        drain_stack(&mut g);
        assert!(g.battlefield_find(small).is_none(), "MV-2 creature destroyed");
    }

    /// Imodane's Recruiter's ETB pumps and hastens your team.
    #[test]
    fn imodanes_recruiter_team_pump_and_haste() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.move_card_to_battlefield_for_test(0, catalog::imodanes_recruiter());
        drain_stack(&mut g);
        let c = g.computed_permanent(bear).unwrap();
        assert_eq!(c.power, 3, "+1/+0 to the team");
        assert!(c.keywords.contains(&crabomination::card::Keyword::Haste), "granted haste");
    }

    /// Elvish Vanguard grows whenever another Elf enters.
    #[test]
    fn elvish_vanguard_grows_on_elf() {
        let mut g = two_player_game();
        let vanguard = g.add_card_to_battlefield(0, catalog::elvish_vanguard());
        let elf = g.add_card_to_battlefield(0, catalog::yevas_forcemage()); // an Elf
        g.dispatch_triggers_for_events(&[GameEvent::PermanentEntered { card_id: elf }]);
        drain_stack(&mut g);
        assert_eq!(
            g.battlefield_find(vanguard).unwrap().counter_count(CounterType::PlusOnePlusOne),
            1,
            "another Elf entering added a counter",
        );
    }

    /// Neutralizing Blast counters a multicolored spell but not a monocolored one.
    #[test]
    fn neutralizing_blast_only_hits_multicolored() {
        let mut g = two_player_game();
        g.step = TurnStep::PreCombatMain;
        // A monocolored creature spell on the stack is not a legal target.
        let mono = g.add_card_to_hand(0, catalog::grizzly_bears());
        let blast = g.add_card_to_hand(1, catalog::neutralizing_blast());
        g.players[0].mana_pool.add(crabomination::mana::Color::Green, 2);
        g.perform_action(GameAction::CastSpell {
            card_id: mono, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast the bear");
        g.players[1].mana_pool.add(crabomination::mana::Color::Blue, 1);
        g.players[1].mana_pool.add_colorless(1);
        g.priority.player_with_priority = 1;
        assert!(
            g.perform_action(GameAction::CastSpell {
                card_id: blast,
                target: Some(Target::Permanent(mono)),
                additional_targets: vec![],
                mode: None,
                x_value: None,
            })
            .is_err(),
            "monocolored spell is not a legal target",
        );
    }

    /// Hoard Robber mints a Treasure on combat damage to a player.
    #[test]
    fn hoard_robber_treasure_on_combat_damage() {
        let mut g = two_player_game();
        g.active_player_idx = 0;
        let robber = g.add_card_to_battlefield(0, catalog::hoard_robber());
        g.clear_sickness(robber);
        advance_to(&mut g, TurnStep::DeclareAttackers);
        g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker: robber,
            target: AttackTarget::Player(1),
        }]))
        .unwrap();
        drain_stack(&mut g);
        advance_to(&mut g, TurnStep::CombatDamage);
        drain_stack(&mut g);
        assert!(
            g.battlefield.iter().any(|c| c.definition.name == "Treasure" && c.controller == 0),
            "combat damage to a player made a Treasure",
        );
    }
}
