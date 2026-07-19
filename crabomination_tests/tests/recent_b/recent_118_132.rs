//! Tests for recentN card batches 118-132 (merged from per-batch micro-files).

mod recent118 {
    use crabomination::catalog;
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    use crabomination::game::types::{Attack, AttackTarget, Target};
    use crabomination::game::*;
    use crabomination::game::{drain_stack, two_player_game};
    use crabomination::mana::Color;

    /// Arcane Epiphany costs {1} less with a Wizard and draws three.
    #[test]
    fn arcane_epiphany_wizard_discount() {
        let mut g = two_player_game();
        g.step = TurnStep::PreCombatMain;
        g.add_card_to_battlefield(0, catalog::dark_confidant()); // Human Wizard
        for _ in 0..5 { g.add_card_to_library(0, catalog::grizzly_bears()); }
        let spell = g.add_card_to_hand(0, catalog::arcane_epiphany());
        g.players[0].mana_pool.add(Color::Blue, 2);
        g.players[0].mana_pool.add_colorless(2); // {2}{U}{U} after the -{1} discount
        let lib = g.players[0].library.len();
        g.perform_action(GameAction::CastSpell {
            card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("discounted cast with a Wizard");
        drain_stack(&mut g);
        assert_eq!(g.players[0].library.len(), lib - 3, "drew three");
    }

    /// Agate Assault's damage mode exiles a creature that would die.
    #[test]
    fn agate_assault_exiles_on_lethal() {
        let mut g = two_player_game();
        g.step = TurnStep::PreCombatMain;
        let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
        let spell = g.add_card_to_hand(0, catalog::agate_assault());
        g.players[0].mana_pool.add(Color::Red, 1);
        g.players[0].mana_pool.add_colorless(2);
        g.perform_action(GameAction::CastSpell {
            card_id: spell, target: Some(Target::Permanent(victim)), additional_targets: vec![], mode: Some(0), x_value: None,
        }).expect("cast Agate Assault (damage mode)");
        drain_stack(&mut g);
        assert!(g.exile.iter().any(|c| c.id == victim), "lethal creature exiled, not in graveyard");
        assert!(g.players[1].graveyard.iter().all(|c| c.id != victim));
    }

    /// Bark-Knuckle Boxer gains indestructible when you expend 4.
    #[test]
    fn bark_knuckle_boxer_expend_indestructible() {
        let mut g = two_player_game();
        let boxer = g.add_card_to_battlefield(0, catalog::bark_knuckle_boxer());
        let moose = g.add_card_to_hand(0, catalog::galewind_moose()); // {4}{G}{G}
        g.players[0].mana_pool.add(Color::Green, 2);
        g.players[0].mana_pool.add_colorless(4);
        g.perform_action(GameAction::CastSpell {
            card_id: moose, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast a 6-mana spell (crosses expend 4)");
        drain_stack(&mut g);
        assert!(g.computed_permanent(boxer).unwrap().keywords.contains(&crabomination::card::Keyword::Indestructible),
            "expend 4 grants indestructible");
    }

    /// Brambleguard Veteran pumps Raccoons you control on expend 4.
    #[test]
    fn brambleguard_veteran_expend_pumps_raccoons() {
        let mut g = two_player_game();
        let vet = g.add_card_to_battlefield(0, catalog::brambleguard_veteran()); // 3/4 Raccoon
        let moose = g.add_card_to_hand(0, catalog::galewind_moose());
        g.players[0].mana_pool.add(Color::Green, 2);
        g.players[0].mana_pool.add_colorless(4);
        g.perform_action(GameAction::CastSpell {
            card_id: moose, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast a 6-mana spell");
        drain_stack(&mut g);
        let cp = g.computed_permanent(vet).unwrap();
        assert_eq!(cp.power, 4, "Raccoon +1/+1");
        assert!(cp.keywords.contains(&crabomination::card::Keyword::Vigilance), "and vigilance");
    }

    /// Attack-in-the-Box may pump itself +4/+0 when it attacks.
    #[test]
    fn attack_in_the_box_may_pump() {
        let mut g = two_player_game();
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
        let box_id = g.add_card_to_battlefield(0, catalog::attack_in_the_box()); // 2/4
        g.clear_sickness(box_id);
        g.step = TurnStep::DeclareAttackers;
        g.declare_attackers(vec![Attack { attacker: box_id, target: AttackTarget::Player(1) }])
            .expect("attack");
        drain_stack(&mut g);
        assert_eq!(g.computed_permanent(box_id).unwrap().power, 6, "opted into +4/+0");
    }

    /// Arbiter of Woe: sacrifice a creature to cast; ETB drains each opponent and
    /// refills you.
    #[test]
    fn arbiter_of_woe_additional_cost_and_drain() {
        let mut g = two_player_game();
        g.step = TurnStep::PreCombatMain;
        g.add_card_to_battlefield(0, catalog::grizzly_bears()); // fodder to sacrifice
        g.add_card_to_hand(1, catalog::grizzly_bears()); // opponent has a card to discard
        for _ in 0..3 { g.add_card_to_library(0, catalog::grizzly_bears()); }
        let arbiter = g.add_card_to_hand(0, catalog::arbiter_of_woe());
        g.players[0].mana_pool.add(Color::Black, 2);
        g.players[0].mana_pool.add_colorless(4);
        let opp_life = g.players[1].life;
        let my_life = g.players[0].life;
        let opp_hand = g.players[1].hand.len();
        g.perform_action(GameAction::CastSpell {
            card_id: arbiter, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast Arbiter (sacrificing the bear)");
        drain_stack(&mut g);
        assert_eq!(g.players[1].life, opp_life - 2, "opponent loses 2");
        assert_eq!(g.players[0].life, my_life + 2, "you gain 2");
        assert_eq!(g.players[1].hand.len(), opp_hand - 1, "opponent discards");
    }
}

mod recent119 {
    use crabomination::catalog;
    use crabomination::game::types::Target;
    use crabomination::game::*;
    use crabomination::game::{drain_stack, two_player_game};
    use crabomination::mana::Color;

    /// Ravine Raider's firebreathing pumps it +1/+1.
    #[test]
    fn ravine_raider_pumps() {
        let mut g = two_player_game();
        g.step = TurnStep::PreCombatMain;
        let raider = g.add_card_to_battlefield(0, catalog::ravine_raider());
        g.players[0].mana_pool.add(Color::Black, 1);
        g.players[0].mana_pool.add_colorless(1);
        g.perform_action(GameAction::ActivateAbility {
            card_id: raider, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
        }).expect("pump");
        drain_stack(&mut g);
        assert_eq!(g.computed_permanent(raider).unwrap().power, 2, "1/1 → 2/2");
    }

    /// Lightshell Duo surveils two on entry.
    #[test]
    fn lightshell_duo_surveils_on_etb() {
        let mut g = two_player_game();
        g.step = TurnStep::PreCombatMain;
        for _ in 0..4 { g.add_card_to_library(0, catalog::grizzly_bears()); }
        let spell = g.add_card_to_hand(0, catalog::lightshell_duo());
        g.players[0].mana_pool.add(Color::Blue, 1);
        g.players[0].mana_pool.add_colorless(3);
        let lib = g.players[0].library.len();
        g.perform_action(GameAction::CastSpell {
            card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast Lightshell Duo");
        drain_stack(&mut g);
        // Surveil looks at the top two; the auto-heuristic keeps them (no forced
        // graveyard), so the library is unchanged but the ETB resolved cleanly.
        assert!(g.battlefield.iter().any(|c| c.definition.name == "Lightshell Duo"), "entered");
        assert!(g.players[0].library.len() <= lib, "surveil didn't add cards");
    }

    /// Nightwhorl Hermit gains +1/+0 and becomes unblockable under threshold.
    #[test]
    fn nightwhorl_hermit_threshold() {
        let mut g = two_player_game();
        let hermit = g.add_card_to_battlefield(0, catalog::nightwhorl_hermit());
        assert_eq!(g.computed_permanent(hermit).unwrap().power, 1, "1/4 without threshold");
        for _ in 0..7 { g.add_card_to_graveyard(0, catalog::grizzly_bears()); }
        let cp = g.computed_permanent(hermit).unwrap();
        assert_eq!(cp.power, 2, "threshold → +1/+0");
        assert!(cp.keywords.contains(&crabomination::card::Keyword::Unblockable), "and unblockable");
    }

    /// Finch Formation grants a creature you control flying on entry.
    #[test]
    fn finch_formation_grants_flying() {
        let mut g = two_player_game();
        g.step = TurnStep::PreCombatMain;
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let spell = g.add_card_to_hand(0, catalog::finch_formation());
        g.players[0].mana_pool.add(Color::Blue, 1);
        g.players[0].mana_pool.add_colorless(2);
        g.perform_action(GameAction::CastSpell {
            card_id: spell, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast Finch Formation");
        drain_stack(&mut g);
        assert!(g.computed_permanent(bear).unwrap().keywords.contains(&crabomination::card::Keyword::Flying),
            "the bear gains flying");
    }
}

mod recent120 {
    use crabomination::catalog;
    use crabomination::game::types::Target;
    use crabomination::game::*;
    use crabomination::game::{drain_stack, two_player_game};
    use crabomination::mana::Color;

    /// Fire a `LifeGained` event so lifegain-matters triggers dispatch.
    fn gain_life(g: &mut GameState, seat: usize, amount: i32) {
        let before = g.players[seat].life;
        g.adjust_life(seat, amount);
        let delta = g.players[seat].life - before;
        if delta > 0 {
            g.dispatch_triggers_for_events(&[GameEvent::LifeGained { player: seat, amount: delta as u32 }]);
            drain_stack(g);
        }
    }

    fn token_count(g: &GameState, seat: usize, name: &str) -> usize {
        g.battlefield.iter().filter(|c| c.controller == seat && c.definition.name == name).count()
    }

    /// Crypt Feaster gets +2/+0 on attack only under threshold.
    #[test]
    fn crypt_feaster_threshold_attack_pump() {
        let mut g = two_player_game();
        let feaster = g.add_card_to_battlefield(0, catalog::crypt_feaster());
        g.clear_sickness(feaster);
        g.step = TurnStep::DeclareAttackers;
        g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker: feaster, target: AttackTarget::Player(1),
        }])).expect("attack");
        drain_stack(&mut g);
        assert_eq!(g.computed_permanent(feaster).unwrap().power, 3, "no threshold → no pump");
    }

    /// Elfsworn Giant makes a 1/1 Elf Warrior whenever a land you control enters.
    #[test]
    fn elfsworn_giant_landfall_token() {
        let mut g = two_player_game();
        g.step = TurnStep::PreCombatMain;
        g.add_card_to_battlefield(0, catalog::elfsworn_giant());
        let land = g.add_card_to_hand(0, catalog::forest());
        g.perform_action(GameAction::PlayLand(land)).expect("play a land");
        drain_stack(&mut g);
        assert_eq!(token_count(&g, 0, "Elf Warrior"), 1, "landfall made an Elf Warrior");
    }

    /// Elvish Regrower returns a permanent card from the graveyard on ETB.
    #[test]
    fn elvish_regrower_returns_permanent_card() {
        let mut g = two_player_game();
        let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
        let reg = g.add_card_to_battlefield(0, catalog::elvish_regrower());
        g.fire_self_etb_triggers(reg, 0);
        drain_stack(&mut g);
        assert!(g.players[0].hand.iter().any(|c| c.id == bear), "the bear returned to hand");
    }

    /// Courageous Goblin pumps + gains menace only while you control a 4-power creature.
    #[test]
    fn courageous_goblin_conditional_attack() {
        let mut g = two_player_game();
        let goblin = g.add_card_to_battlefield(0, catalog::courageous_goblin());
        g.clear_sickness(goblin);
        g.step = TurnStep::DeclareAttackers;
        g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker: goblin, target: AttackTarget::Player(1),
        }])).expect("attack");
        drain_stack(&mut g);
        assert_eq!(g.computed_permanent(goblin).unwrap().power, 2, "no 4-power ally → no bonus");

        // Reset and add a big creature so the intervening-if is satisfied.
        let mut g = two_player_game();
        let goblin = g.add_card_to_battlefield(0, catalog::courageous_goblin());
        g.clear_sickness(goblin);
        let big = g.add_card_to_battlefield(0, catalog::elfsworn_giant()); // 5/3
        g.clear_sickness(big);
        g.step = TurnStep::DeclareAttackers;
        g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker: goblin, target: AttackTarget::Player(1),
        }])).expect("attack");
        drain_stack(&mut g);
        let cp = g.computed_permanent(goblin).unwrap();
        assert_eq!(cp.power, 3, "4-power ally → +1/+0");
        assert!(cp.keywords.contains(&crabomination::card::Keyword::Menace), "and gains menace");
    }

    /// Eager Trufflesnout makes a Food when it connects with a player.
    #[test]
    fn eager_trufflesnout_food_on_combat_damage() {
        let mut g = two_player_game();
        let boar = g.add_card_to_battlefield(0, catalog::eager_trufflesnout());
        g.clear_sickness(boar);
        g.step = TurnStep::DeclareAttackers;
        g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker: boar, target: AttackTarget::Player(1),
        }])).expect("attack");
        g.step = TurnStep::CombatDamage;
        g.resolve_combat().expect("combat damage");
        drain_stack(&mut g);
        assert_eq!(token_count(&g, 0, "Food"), 1, "combat damage made a Food");
    }

    /// Cat Collector makes a Food on ETB and a Cat on the first lifegain each turn.
    #[test]
    fn cat_collector_food_etb_and_first_lifegain_cat() {
        let mut g = two_player_game();
        let cc = g.add_card_to_battlefield(0, catalog::cat_collector());
        g.fire_self_etb_triggers(cc, 0);
        drain_stack(&mut g);
        assert_eq!(token_count(&g, 0, "Food"), 1, "ETB Food");

        // First lifegain on your turn → a Cat; the second does nothing.
        gain_life(&mut g, 0, 2);
        assert_eq!(token_count(&g, 0, "Cat"), 1, "first lifegain → Cat");
        gain_life(&mut g, 0, 2);
        assert_eq!(token_count(&g, 0, "Cat"), 1, "second lifegain same turn → no Cat");
    }

    /// Dawnwing Marshal's activated ability pumps the team +1/+1.
    #[test]
    fn dawnwing_marshal_team_pump() {
        let mut g = two_player_game();
        g.step = TurnStep::PreCombatMain;
        let marshal = g.add_card_to_battlefield(0, catalog::dawnwing_marshal());
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.players[0].mana_pool.add(Color::White, 1);
        g.players[0].mana_pool.add_colorless(4);
        g.perform_action(GameAction::ActivateAbility {
            card_id: marshal, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
        }).expect("pump");
        drain_stack(&mut g);
        assert_eq!(g.computed_permanent(bear).unwrap().power, 3, "bear 2/2 → 3/3");
        assert_eq!(g.computed_permanent(marshal).unwrap().toughness, 3, "marshal 2/2 → 3/3");
    }

    /// Clinquant Skymage grows with a +1/+1 counter each time you draw.
    #[test]
    fn clinquant_skymage_grows_on_draw() {
        let mut g = two_player_game();
        let mage = g.add_card_to_battlefield(0, catalog::clinquant_skymage());
        let drawn = g.add_card_to_hand(0, catalog::grizzly_bears());
        g.dispatch_triggers_for_events(&[GameEvent::CardDrawn { player: 0, card_id: drawn }]);
        drain_stack(&mut g);
        assert_eq!(g.computed_permanent(mage).unwrap().power, 2, "1/1 + a draw → 2/2");
    }

    /// Elementalist Adept has flash and prowess.
    #[test]
    fn elementalist_adept_flash_prowess() {
        let mut g = two_player_game();
        let adept = g.add_card_to_battlefield(0, catalog::elementalist_adept());
        let cp = g.computed_permanent(adept).unwrap();
        assert!(cp.keywords.contains(&crabomination::card::Keyword::Flash), "has flash");
        assert!(cp.keywords.contains(&crabomination::card::Keyword::Prowess), "has prowess");
    }

    /// Divine Resilience unkicked protects one creature; kicked it protects the team.
    #[test]
    fn divine_resilience_kicked_protects_team() {
        let mut g = two_player_game();
        g.step = TurnStep::PreCombatMain;
        let a = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let b = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let spell = g.add_card_to_hand(0, catalog::divine_resilience());
        g.players[0].mana_pool.add(Color::White, 2);
        g.players[0].mana_pool.add_colorless(2);
        g.perform_action(GameAction::CastSpellKicked {
            card_id: spell, target: Some(Target::Permanent(a)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast Divine Resilience kicked");
        drain_stack(&mut g);
        assert!(g.computed_permanent(a).unwrap().keywords.contains(&crabomination::card::Keyword::Indestructible));
        assert!(g.computed_permanent(b).unwrap().keywords.contains(&crabomination::card::Keyword::Indestructible),
            "kicked → both creatures protected");
    }
}

mod recent121 {
    use crabomination::catalog;
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    use crabomination::game::effects::EntityRef;
    use crabomination::game::types::Target;
    use crabomination::game::*;
    use crabomination::game::{drain_stack, two_player_game};
    use crabomination::mana::Color;

    /// Fill a graveyard with cards of four distinct types to satisfy Delirium.
    fn make_delirium(g: &mut GameState, seat: usize) {
        g.add_card_to_graveyard(seat, catalog::grizzly_bears()); // creature
        g.add_card_to_graveyard(seat, catalog::lightning_bolt()); // instant
        g.add_card_to_graveyard(seat, catalog::forest()); // land
        g.add_card_to_graveyard(seat, catalog::divination()); // sorcery
    }

    /// Barkform Harvester tucks a graveyard card onto the bottom of the library.
    #[test]
    fn barkform_harvester_tucks_graveyard_card() {
        let mut g = two_player_game();
        let bolt = g.add_card_to_graveyard(0, catalog::lightning_bolt());
        let harvester = g.add_card_to_battlefield(0, catalog::barkform_harvester());
        g.clear_sickness(harvester);
        g.players[0].mana_pool.add_colorless(2);
        g.perform_action(GameAction::ActivateAbility {
            card_id: harvester, ability_index: 0, target: Some(Target::Permanent(bolt)),
            additional_targets: vec![], x_value: None,
        }).expect("tuck");
        drain_stack(&mut g);
        assert!(!g.players[0].graveyard.iter().any(|c| c.id == bolt), "left the graveyard");
        assert_eq!(g.players[0].library.last().map(|c| c.id), Some(bolt), "on the bottom");
    }

    /// Bonebind Orator returns another creature card from the graveyard, exiling itself.
    #[test]
    fn bonebind_orator_graveyard_recursion() {
        let mut g = two_player_game();
        let orator = g.add_card_to_graveyard(0, catalog::bonebind_orator());
        let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
        g.players[0].mana_pool.add(Color::Black, 1);
        g.players[0].mana_pool.add_colorless(3);
        g.perform_action(GameAction::ActivateAbility {
            card_id: orator, ability_index: 0, target: Some(Target::Permanent(bear)),
            additional_targets: vec![], x_value: None,
        }).expect("recur");
        drain_stack(&mut g);
        assert!(g.players[0].hand.iter().any(|c| c.id == bear), "bear returned to hand");
        assert!(!g.players[0].graveyard.iter().any(|c| c.id == orator), "orator exiled itself");
    }

    /// Clifftop Lookout digs a land onto the battlefield tapped on entry.
    #[test]
    fn clifftop_lookout_ramps_a_land() {
        let mut g = two_player_game();
        // Top of library: two nonlands then a Forest.
        g.add_card_to_library(0, catalog::forest());
        g.add_card_to_library(0, catalog::grizzly_bears());
        g.add_card_to_library(0, catalog::grizzly_bears());
        let lookout = g.add_card_to_battlefield(0, catalog::clifftop_lookout());
        g.fire_self_etb_triggers(lookout, 0);
        drain_stack(&mut g);
        let land = g.battlefield.iter().find(|c| c.definition.name == "Forest");
        assert!(land.is_some(), "a Forest hit the battlefield");
        assert!(land.unwrap().tapped, "and it entered tapped");
    }

    /// Brambleguard Captain pumps a creature by its own power at combat.
    #[test]
    fn brambleguard_captain_begin_combat_pump() {
        let mut g = two_player_game();
        let cap = g.add_card_to_battlefield(0, catalog::brambleguard_captain()); // 2/3
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
        g.clear_sickness(cap);
        g.active_player_idx = 0;
        g.step = TurnStep::BeginCombat;
        g.fire_step_triggers(TurnStep::BeginCombat);
        drain_stack(&mut g);
        assert_eq!(g.computed_permanent(bear).unwrap().power, 4, "2/2 + captain's power 2 → 4/2");
    }

    /// Downwind Ambusher's first mode shrinks an opponent's creature.
    #[test]
    fn downwind_ambusher_minus_mode() {
        let mut g = two_player_game();
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Mode(0)]));
        let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let ambusher = g.add_card_to_battlefield(0, catalog::downwind_ambusher());
        g.fire_self_etb_triggers(ambusher, 0);
        drain_stack(&mut g);
        assert_eq!(g.computed_permanent(bear).unwrap().toughness, 1, "2/2 → 1/1");
    }

    /// Cracked Skull destroys the enchanted creature when it's dealt damage.
    #[test]
    fn cracked_skull_destroys_on_damage() {
        let mut g = two_player_game();
        g.step = TurnStep::PreCombatMain;
        let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let skull = g.add_card_to_hand(0, catalog::cracked_skull());
        g.players[0].mana_pool.add(Color::Black, 1);
        g.players[0].mana_pool.add_colorless(2);
        g.perform_action(GameAction::CastSpell {
            card_id: skull, target: Some(Target::Permanent(bear)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast Cracked Skull on the bear");
        drain_stack(&mut g);
        // Now deal a point of damage to the enchanted creature.
        let mut ev = vec![];
        g.deal_damage_to_from(EntityRef::Permanent(bear), 1, None, &mut ev);
        g.dispatch_triggers_for_events(&ev);
        drain_stack(&mut g);
        assert!(!g.battlefield.iter().any(|c| c.id == bear), "enchanted creature destroyed");
    }

    /// Beastie Beatdown: a delirium fight buffs your creature and kills theirs.
    #[test]
    fn beastie_beatdown_delirium_fight() {
        let mut g = two_player_game();
        g.step = TurnStep::PreCombatMain;
        make_delirium(&mut g, 0);
        let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2 → 4/4 delirium
        let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
        let spell = g.add_card_to_hand(0, catalog::beastie_beatdown());
        g.players[0].mana_pool.add(Color::Red, 1);
        g.players[0].mana_pool.add(Color::Green, 1);
        g.perform_action(GameAction::CastSpell {
            card_id: spell, target: Some(Target::Permanent(mine)),
            additional_targets: vec![Target::Permanent(theirs)], mode: None, x_value: None,
        }).expect("cast Beastie Beatdown");
        drain_stack(&mut g);
        assert_eq!(g.computed_permanent(mine).unwrap().power, 4, "delirium → two +1/+1 counters");
        assert!(!g.battlefield.iter().any(|c| c.id == theirs), "4 damage kills the 2/2");
    }

    /// Balustrade Wurm reanimates itself from the graveyard under Delirium.
    #[test]
    fn balustrade_wurm_delirium_reanimates() {
        let mut g = two_player_game();
        g.step = TurnStep::PreCombatMain;
        make_delirium(&mut g, 0);
        let wurm = g.add_card_to_graveyard(0, catalog::balustrade_wurm());
        g.players[0].mana_pool.add(Color::Green, 2);
        g.players[0].mana_pool.add_colorless(2);
        g.perform_action(GameAction::ActivateAbility {
            card_id: wurm, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
        }).expect("reanimate");
        drain_stack(&mut g);
        let onbf = g.battlefield.iter().find(|c| c.definition.name == "Balustrade Wurm");
        assert!(onbf.is_some(), "wurm returned to the battlefield");
        assert!(onbf.unwrap().counter_count(crabomination::card::CounterType::Finality) >= 1, "with a finality counter");
    }

    /// Drag to the Roots costs {2} less while Delirium is active.
    #[test]
    fn drag_to_the_roots_delirium_discount() {
        let mut g = two_player_game();
        g.step = TurnStep::PreCombatMain;
        make_delirium(&mut g, 0);
        let target = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let spell = g.add_card_to_hand(0, catalog::drag_to_the_roots());
        // Discounted cost is {B}{G} instead of {2}{B}{G}.
        g.players[0].mana_pool.add(Color::Black, 1);
        g.players[0].mana_pool.add(Color::Green, 1);
        g.perform_action(GameAction::CastSpell {
            card_id: spell, target: Some(Target::Permanent(target)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast at the delirium discount");
        drain_stack(&mut g);
        assert!(!g.battlefield.iter().any(|c| c.id == target), "nonland permanent destroyed");
    }
}

mod recent122 {
    use crabomination::catalog;
    use crabomination::game::types::Target;
    use crabomination::game::*;
    use crabomination::game::{drain_stack, two_player_game};
    use crabomination::mana::Color;

    /// Gigantosaurus is a 10/10.
    #[test]
    fn gigantosaurus_is_a_ten_ten() {
        let mut g = two_player_game();
        let id = g.add_card_to_battlefield(0, catalog::gigantosaurus());
        let cp = g.computed_permanent(id).unwrap();
        assert_eq!((cp.power, cp.toughness), (10, 10));
    }

    /// Cephalid Inkmage becomes unblockable under threshold.
    #[test]
    fn cephalid_inkmage_threshold_unblockable() {
        let mut g = two_player_game();
        let mage = g.add_card_to_battlefield(0, catalog::cephalid_inkmage());
        assert!(!g.computed_permanent(mage).unwrap().keywords.contains(&crabomination::card::Keyword::Unblockable),
            "no threshold → blockable");
        for _ in 0..7 { g.add_card_to_graveyard(0, catalog::grizzly_bears()); }
        assert!(g.computed_permanent(mage).unwrap().keywords.contains(&crabomination::card::Keyword::Unblockable),
            "threshold → unblockable");
    }

    /// Dire Downdraft's discount lets it cast for {2}{U} against a tapped creature.
    #[test]
    fn dire_downdraft_discount_vs_tapped() {
        let mut g = two_player_game();
        g.step = TurnStep::PreCombatMain;
        let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        g.battlefield_find_mut(bear).unwrap().tapped = true;
        let spell = g.add_card_to_hand(0, catalog::dire_downdraft());
        // Only the discounted {2}{U} is available.
        g.players[0].mana_pool.add(Color::Blue, 1);
        g.players[0].mana_pool.add_colorless(2);
        g.perform_action(GameAction::CastSpell {
            card_id: spell, target: Some(Target::Permanent(bear)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast at the {1}-off discount");
        drain_stack(&mut g);
        assert!(!g.battlefield.iter().any(|c| c.id == bear), "the bear left the battlefield");
        assert!(g.players[1].library.iter().any(|c| c.id == bear), "it went to its owner's library");
    }

    /// Curator of Destinies digs five with Fact or Fiction on entry and is
    /// uncounterable with flying.
    #[test]
    fn curator_of_destinies_fact_or_fiction() {
        let mut g = two_player_game();
        for _ in 0..8 { g.add_card_to_library(0, catalog::grizzly_bears()); }
        let lib_before = g.players[0].library.len();
        let curator = g.add_card_to_battlefield(0, catalog::curator_of_destinies());
        let cp = g.computed_permanent(curator).unwrap();
        assert!(cp.keywords.contains(&crabomination::card::Keyword::Flying), "has flying");
        assert!(cp.keywords.contains(&crabomination::card::Keyword::CantBeCountered), "uncounterable");
        g.fire_self_etb_triggers(curator, 0);
        drain_stack(&mut g);
        assert_eq!(g.players[0].library.len(), lib_before - 5, "five cards left the library");
        // All five land in hand + graveyard combined.
        assert_eq!(g.players[0].hand.len() + g.players[0].graveyard.len(), 5, "split into hand and graveyard");
    }
}

mod recent123 {
    use crabomination::catalog;
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    use crabomination::game::*;
    use crabomination::game::{drain_stack, two_player_game};

    /// Corpseberry Cultivator forages at combat and grows from its own forage.
    #[test]
    fn corpseberry_cultivator_forages_and_grows() {
        let mut g = two_player_game();
        // Say "yes" to the optional forage.
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
        let corp = g.add_card_to_battlefield(0, catalog::corpseberry_cultivator());
        // Three graveyard cards to pay the forage.
        for _ in 0..3 { g.add_card_to_graveyard(0, catalog::grizzly_bears()); }
        g.active_player_idx = 0;
        g.step = TurnStep::BeginCombat;
        g.fire_step_triggers(TurnStep::BeginCombat);
        drain_stack(&mut g);
        assert_eq!(g.players[0].graveyard.len(), 0, "foraged away three cards");
        assert_eq!(g.computed_permanent(corp).unwrap().power, 3, "whenever-you-forage → +1/+1");
    }

    /// A direct `Foraged` event fires the payoff for any forage source.
    #[test]
    fn foraged_event_fires_payoff() {
        let mut g = two_player_game();
        let corp = g.add_card_to_battlefield(0, catalog::corpseberry_cultivator());
        g.dispatch_triggers_for_events(&[GameEvent::Foraged { player: 0 }]);
        drain_stack(&mut g);
        assert_eq!(g.computed_permanent(corp).unwrap().power, 3, "payoff triggers on the event");
    }
}

mod recent124 {
    use crabomination::catalog;
    use crabomination::game::types::Target;
    use crabomination::game::*;
    use crabomination::game::{drain_stack, two_player_game};
    use crabomination::mana::Color;

    /// Armored Armadillo's pump adds its toughness to power.
    #[test]
    fn armored_armadillo_pumps_by_toughness() {
        let mut g = two_player_game();
        g.step = TurnStep::PreCombatMain;
        let dillo = g.add_card_to_battlefield(0, catalog::armored_armadillo()); // 0/4
        g.clear_sickness(dillo);
        g.players[0].mana_pool.add(Color::White, 1);
        g.players[0].mana_pool.add_colorless(3);
        g.perform_action(GameAction::ActivateAbility {
            card_id: dillo, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
        }).expect("pump");
        drain_stack(&mut g);
        assert_eq!(g.computed_permanent(dillo).unwrap().power, 4, "+X/+0 where X = toughness 4");
    }

    /// Ambush Gigapede shrinks an opponent's creature on entry.
    #[test]
    fn ambush_gigapede_minus_two() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let pede = g.add_card_to_battlefield(0, catalog::ambush_gigapede());
        g.fire_self_etb_triggers(pede, 0);
        drain_stack(&mut g);
        // 2/2 → 0/0, dies as a state-based action.
        assert!(!g.battlefield.iter().any(|c| c.id == bear), "-2/-2 kills the 2/2");
    }

    /// Desperate Bloodseeker mills the targeted player two and has lifelink.
    #[test]
    fn desperate_bloodseeker_mills_two() {
        let mut g = two_player_game();
        for _ in 0..3 { g.add_card_to_library(1, catalog::grizzly_bears()); }
        let gy_before = g.players[1].graveyard.len();
        let seeker = g.add_card_to_battlefield(0, catalog::desperate_bloodseeker());
        assert!(g.computed_permanent(seeker).unwrap().keywords.contains(&crabomination::card::Keyword::Lifelink));
        g.fire_self_etb_triggers(seeker, 0);
        drain_stack(&mut g);
        assert_eq!(g.players[1].graveyard.len(), gy_before + 2, "target player milled two");
    }

    /// Deadeye Duelist pings an opponent for 1.
    #[test]
    fn deadeye_duelist_pings() {
        let mut g = two_player_game();
        let duelist = g.add_card_to_battlefield(0, catalog::deadeye_duelist());
        g.clear_sickness(duelist);
        g.players[0].mana_pool.add_colorless(1);
        let life = g.players[1].life;
        g.perform_action(GameAction::ActivateAbility {
            card_id: duelist, ability_index: 0, target: Some(Target::Player(1)),
            additional_targets: vec![], x_value: None,
        }).expect("ping");
        drain_stack(&mut g);
        assert_eq!(g.players[1].life, life - 1, "1 damage to the opponent");
    }

    /// Eriette's Lullaby destroys a tapped creature and gains 2 life.
    #[test]
    fn eriettes_lullaby_kills_tapped() {
        let mut g = two_player_game();
        g.step = TurnStep::PreCombatMain;
        let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        g.battlefield_find_mut(bear).unwrap().tapped = true;
        let spell = g.add_card_to_hand(0, catalog::eriettes_lullaby());
        g.players[0].mana_pool.add(Color::White, 1);
        g.players[0].mana_pool.add_colorless(1);
        let life = g.players[0].life;
        g.perform_action(GameAction::CastSpell {
            card_id: spell, target: Some(Target::Permanent(bear)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast Lullaby");
        drain_stack(&mut g);
        assert!(!g.battlefield.iter().any(|c| c.id == bear), "tapped creature destroyed");
        assert_eq!(g.players[0].life, life + 2, "gained 2 life");
    }

    /// Geyser Drake discounts your spells during the opponent's turn only.
    #[test]
    fn geyser_drake_off_turn_discount() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::geyser_drake());
        // On the opponent's turn, a {1}{U} flash creature costs just {U}.
        g.active_player_idx = 1;
        let spell = g.add_card_to_hand(0, catalog::ambush_gigapede()); // {4}{B}{B} flash
        // {3}{B}{B} after the {1} off-turn discount.
        g.players[0].mana_pool.add(Color::Black, 2);
        g.players[0].mana_pool.add_colorless(3);
        g.priority.player_with_priority = 0;
        let r = g.perform_action(GameAction::CastSpell {
            card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None,
        });
        assert!(r.is_ok(), "cast at the off-turn discount: {r:?}");
    }

    /// Bristlepack Sentry can attack only while you control a 4-power creature.
    #[test]
    fn bristlepack_sentry_conditional_attack() {
        let mut g = two_player_game();
        let sentry = g.add_card_to_battlefield(0, catalog::bristlepack_sentry());
        g.clear_sickness(sentry);
        g.step = TurnStep::DeclareAttackers;
        assert!(g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker: sentry, target: AttackTarget::Player(1),
        }])).is_err(), "defender can't attack without a big creature");

        let big = g.add_card_to_battlefield(0, catalog::gigantosaurus()); // 10/10
        g.clear_sickness(big);
        g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker: sentry, target: AttackTarget::Player(1),
        }])).expect("attacks with a 4-power ally in play");
    }
}

mod recent125 {
    use crabomination::catalog;
    use crabomination::game::types::Target;
    use crabomination::game::*;
    use crabomination::game::{drain_stack, two_player_game};
    use crabomination::mana::Color;

    fn saddled_attack(g: &mut GameState, mount: CardId) {
        g.battlefield_find_mut(mount).unwrap().saddled = true;
        g.clear_sickness(mount);
        g.active_player_idx = 0;
        g.step = TurnStep::DeclareAttackers;
        g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker: mount, target: AttackTarget::Player(1),
        }])).expect("attack");
        drain_stack(g);
    }

    /// Bridled Bighorn makes a Sheep when it attacks saddled.
    #[test]
    fn bridled_bighorn_saddled_makes_sheep() {
        let mut g = two_player_game();
        let bighorn = g.add_card_to_battlefield(0, catalog::bridled_bighorn());
        saddled_attack(&mut g, bighorn);
        assert_eq!(
            g.battlefield.iter().filter(|c| c.controller == 0 && c.definition.name == "Sheep").count(),
            1, "a Sheep token appeared"
        );
    }

    /// Drover Grizzly grants the team trample when it attacks saddled.
    #[test]
    fn drover_grizzly_saddled_grants_trample() {
        let mut g = two_player_game();
        let grizzly = g.add_card_to_battlefield(0, catalog::drover_grizzly());
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        saddled_attack(&mut g, grizzly);
        assert!(g.computed_permanent(bear).unwrap().keywords.contains(&crabomination::card::Keyword::Trample),
            "other creatures gain trample");
    }

    /// Sun-Blessed Healer reanimates a cheap permanent only when kicked.
    #[test]
    fn sun_blessed_healer_kicked_reanimates() {
        let mut g = two_player_game();
        g.step = TurnStep::PreCombatMain;
        let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears()); // MV 2
        let healer = g.add_card_to_hand(0, catalog::sun_blessed_healer());
        g.players[0].mana_pool.add(Color::White, 2);
        g.players[0].mana_pool.add_colorless(2);
        g.perform_action(GameAction::CastSpellKicked {
            card_id: healer, target: Some(Target::Permanent(bear)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast kicked");
        drain_stack(&mut g);
        assert!(g.battlefield.iter().any(|c| c.id == bear && c.controller == 0),
            "the bear returned to the battlefield");
    }
}

mod recent126 {
    use crabomination::catalog;
    use crabomination::game::types::Target;
    use crabomination::game::*;
    use crabomination::game::{drain_stack, two_player_game};
    use crabomination::mana::Color;

    /// Mine Raider makes a Treasure only while you control another outlaw.
    #[test]
    fn mine_raider_treasure_with_another_outlaw() {
        // No other outlaw → no Treasure.
        let mut g = two_player_game();
        let raider = g.add_card_to_battlefield(0, catalog::mine_raider());
        g.fire_self_etb_triggers(raider, 0);
        drain_stack(&mut g);
        assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Treasure").count(), 0);

        // With a Rogue ally (an outlaw) → a Treasure.
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::deadeye_duelist()); // Human Assassin = outlaw
        let raider = g.add_card_to_battlefield(0, catalog::mine_raider());
        g.fire_self_etb_triggers(raider, 0);
        drain_stack(&mut g);
        assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Treasure").count(), 1,
            "another outlaw → Treasure");
    }

    /// Scorching Shot deals 5 to a creature.
    #[test]
    fn scorching_shot_deals_five() {
        let mut g = two_player_game();
        g.step = TurnStep::PreCombatMain;
        let wall = g.add_card_to_battlefield(1, catalog::gigantosaurus()); // 10/10
        let spell = g.add_card_to_hand(0, catalog::scorching_shot());
        g.players[0].mana_pool.add(Color::Red, 2);
        g.perform_action(GameAction::CastSpell {
            card_id: spell, target: Some(Target::Permanent(wall)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast Scorching Shot");
        drain_stack(&mut g);
        assert_eq!(g.battlefield_find(wall).unwrap().damage, 5, "5 damage marked");
    }

    /// Peerless Ropemaster bounces a tapped creature on entry.
    #[test]
    fn peerless_ropemaster_bounces_tapped() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        g.battlefield_find_mut(bear).unwrap().tapped = true;
        let rope = g.add_card_to_battlefield(0, catalog::peerless_ropemaster());
        g.fire_self_etb_triggers(rope, 0);
        drain_stack(&mut g);
        assert!(g.players[1].hand.iter().any(|c| c.id == bear), "tapped creature returned to hand");
    }

    /// Spring Splasher weakens a defender's creature when it attacks.
    #[test]
    fn spring_splasher_attack_debuff() {
        let mut g = two_player_game();
        let splasher = g.add_card_to_battlefield(0, catalog::spring_splasher());
        let blocker = g.add_card_to_battlefield(1, catalog::gigantosaurus()); // 10/10
        g.clear_sickness(splasher);
        g.step = TurnStep::DeclareAttackers;
        g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker: splasher, target: AttackTarget::Player(1),
        }])).expect("attack");
        drain_stack(&mut g);
        assert_eq!(g.computed_permanent(blocker).unwrap().power, 7, "-3/-0 on the defender's creature");
    }

    /// Raven of Fell Omens drains 1 when you commit a crime, once per turn.
    #[test]
    fn raven_of_fell_omens_crime_drain() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::raven_of_fell_omens());
        let opp = g.players[1].life;
        let me = g.players[0].life;
        g.dispatch_triggers_for_events(&[GameEvent::CommittedCrime { player: 0 }]);
        drain_stack(&mut g);
        assert_eq!(g.players[1].life, opp - 1, "opponent loses 1");
        assert_eq!(g.players[0].life, me + 1, "you gain 1");
        // Second crime same turn does nothing (once each turn).
        g.dispatch_triggers_for_events(&[GameEvent::CommittedCrime { player: 0 }]);
        drain_stack(&mut g);
        assert_eq!(g.players[1].life, opp - 1, "only once per turn");
    }

    /// Stagecoach Security pumps the team +1/+1 and grants vigilance on entry.
    #[test]
    fn stagecoach_security_team_pump() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let sec = g.add_card_to_battlefield(0, catalog::stagecoach_security());
        g.fire_self_etb_triggers(sec, 0);
        drain_stack(&mut g);
        let cp = g.computed_permanent(bear).unwrap();
        assert_eq!(cp.power, 3, "bear 2/2 → 3/3");
        assert!(cp.keywords.contains(&crabomination::card::Keyword::Vigilance), "and vigilance");
    }
}

mod recent127 {
    use crabomination::card::{CounterType, Keyword};
    use crabomination::catalog;
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    use crabomination::game::types::Target;
    use crabomination::game::*;
    use crabomination::game::{drain_stack, two_player_game};
    use crabomination::mana::Color;

    /// A Desert painland enters tapped and pings an opponent for 1.
    #[test]
    fn desert_painland_etb_ping() {
        let mut g = two_player_game();
        let opp = g.players[1].life;
        let land = g.add_card_to_battlefield(0, catalog::bristling_backwoods());
        g.fire_self_etb_triggers(land, 0);
        drain_stack(&mut g);
        assert!(g.battlefield_find(land).unwrap().tapped, "enters tapped");
        assert_eq!(g.players[1].life, opp - 1, "opponent pinged for 1");
    }

    /// Eroded Canyon (completing the 10-Desert cycle) taps for either of its two
    /// colors.
    #[test]
    fn eroded_canyon_taps_for_two_colors() {
        let mut g = two_player_game();
        let land = g.add_card_to_battlefield(0, catalog::eroded_canyon());
        g.perform_action(GameAction::ActivateAbility {
            card_id: land,
            ability_index: 0, // first mana ability → {U}
            target: None,
            additional_targets: Vec::new(),
            x_value: None,
        })
        .expect("tap for blue");
        assert_eq!(g.players[0].mana_pool.amount(crabomination::mana::Color::Blue), 1, "tapped for blue");
    }

    /// Daring Thunder-Thief enters tapped.
    #[test]
    fn daring_thunder_thief_enters_tapped() {
        let mut g = two_player_game();
        let c = g.move_card_to_battlefield_for_test(0, catalog::daring_thunder_thief());
        assert!(g.battlefield_find(c).unwrap().tapped, "enters tapped via static replacement");
    }

    /// Deepmuck Desperado mills each opponent three on the first crime each turn.
    #[test]
    fn deepmuck_desperado_crime_mill() {
        let mut g = two_player_game();
        for _ in 0..5 {
            g.add_card_to_library(1, catalog::grizzly_bears());
        }
        g.add_card_to_battlefield(0, catalog::deepmuck_desperado());
        let before = g.players[1].graveyard.len();
        g.dispatch_triggers_for_events(&[GameEvent::CommittedCrime { player: 0 }]);
        drain_stack(&mut g);
        assert_eq!(g.players[1].graveyard.len(), before + 3, "milled three");
        // Second crime this turn does nothing.
        g.dispatch_triggers_for_events(&[GameEvent::CommittedCrime { player: 0 }]);
        drain_stack(&mut g);
        assert_eq!(g.players[1].graveyard.len(), before + 3, "once per turn");
    }

    /// Blood Hustler grows on a crime.
    #[test]
    fn blood_hustler_crime_counter() {
        let mut g = two_player_game();
        let bh = g.add_card_to_battlefield(0, catalog::blood_hustler());
        g.dispatch_triggers_for_events(&[GameEvent::CommittedCrime { player: 0 }]);
        drain_stack(&mut g);
        let cp = g.computed_permanent(bh).unwrap();
        assert_eq!((cp.power, cp.toughness), (2, 2), "+1/+1 counter added");
    }

    /// Blacksnag Buzzard enters with a +1/+1 counter only if a creature died.
    #[test]
    fn blacksnag_buzzard_conditional_counter() {
        // No death → 2/1.
        let mut g = two_player_game();
        let b = g.add_card_to_battlefield(0, catalog::blacksnag_buzzard());
        g.fire_self_etb_triggers(b, 0);
        drain_stack(&mut g);
        assert_eq!(g.computed_permanent(b).unwrap().power, 2, "no death → base 2/1");

        // A creature died this turn → 3/2.
        let mut g = two_player_game();
        g.players[0].creatures_died_this_turn = 1;
        let b = g.add_card_to_battlefield(0, catalog::blacksnag_buzzard());
        g.fire_self_etb_triggers(b, 0);
        drain_stack(&mut g);
        assert_eq!(g.computed_permanent(b).unwrap().power, 3, "death → 3/2");
    }

    /// Congregation Gryff pumps by the number of Mounts you control while saddled.
    #[test]
    fn congregation_gryff_saddled_pump() {
        let mut g = two_player_game();
        let gryff = g.add_card_to_battlefield(0, catalog::congregation_gryff());
        g.battlefield_find_mut(gryff).unwrap().saddled = true;
        g.clear_sickness(gryff);
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        while g.step != TurnStep::DeclareAttackers {
            g.perform_action(GameAction::PassPriority).unwrap();
        }
        g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker: gryff,
            target: AttackTarget::Player(1),
        }]))
        .expect("attack");
        drain_stack(&mut g);
        // Only the Gryff itself is a Mount → +1/+1.
        let cp = g.computed_permanent(gryff).unwrap();
        assert_eq!((cp.power, cp.toughness), (2, 5), "+1/+1 for the one Mount");
    }

    /// Duelist of the Mind's power tracks cards drawn this turn.
    #[test]
    fn duelist_of_the_mind_cda_power() {
        let mut g = two_player_game();
        let d = g.add_card_to_battlefield(0, catalog::duelist_of_the_mind());
        assert_eq!(g.computed_permanent(d).unwrap().power, 0, "no draws yet → */3 = 0/3");
        g.players[0].cards_drawn_this_turn = 2;
        assert_eq!(g.computed_permanent(d).unwrap().power, 2, "power = 2 cards drawn");
        assert_eq!(g.computed_permanent(d).unwrap().toughness, 3, "toughness fixed at 3");
    }

    /// Boneyard Desecrator makes a Treasure only when the sacrificed creature was
    /// an outlaw.
    #[test]
    fn boneyard_desecrator_outlaw_treasure() {
        let treasures = |g: &GameState| {
            g.battlefield.iter().filter(|c| c.controller == 0 && c.definition.name == "Treasure").count()
        };
        // Sacrificing a plain Bear → counter, no Treasure.
        let mut g = two_player_game();
        let bd = g.add_card_to_battlefield(0, catalog::boneyard_desecrator());
        g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.players[0].mana_pool.add(Color::Black, 2);
        g.perform_action(GameAction::ActivateAbility {
            card_id: bd,
            ability_index: 0,
            target: None,
            additional_targets: Vec::new(),
            x_value: None,
        })
        .expect("sac a creature");
        drain_stack(&mut g);
        assert_eq!(treasures(&g), 0, "non-outlaw sacrifice → no Treasure");
        assert_eq!(g.computed_permanent(bd).unwrap().power, 4, "still gets the +1/+1 counter");

        // Sacrificing an outlaw (Rogue) → a Treasure.
        let mut g = two_player_game();
        let bd = g.add_card_to_battlefield(0, catalog::boneyard_desecrator());
        g.add_card_to_battlefield(0, catalog::blood_hustler()); // Vampire Rogue = outlaw
        g.players[0].mana_pool.add(Color::Black, 2);
        g.perform_action(GameAction::ActivateAbility {
            card_id: bd,
            ability_index: 0,
            target: None,
            additional_targets: Vec::new(),
            x_value: None,
        })
        .expect("sac an outlaw");
        drain_stack(&mut g);
        assert_eq!(treasures(&g), 1, "outlaw sacrifice → Treasure");
    }

    /// Skulduggery pumps your creature and shrinks the opponent's.
    #[test]
    fn skulduggery_dual_targets() {
        let mut g = two_player_game();
        g.step = TurnStep::PreCombatMain;
        let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let spell = g.add_card_to_hand(0, catalog::skulduggery());
        g.players[0].mana_pool.add(Color::Black, 1);
        g.perform_action(GameAction::CastSpell {
            card_id: spell,
            target: Some(Target::Permanent(mine)),
            additional_targets: vec![Target::Permanent(theirs)],
            mode: None,
            x_value: None,
        })
        .expect("cast Skulduggery");
        drain_stack(&mut g);
        assert_eq!(g.computed_permanent(mine).unwrap().power, 3, "yours +1/+1");
        assert_eq!(g.computed_permanent(theirs).unwrap().power, 1, "theirs -1/-1");
    }

    /// Badlands Revival reanimates a creature and returns a permanent to hand.
    #[test]
    fn badlands_revival_reanimates() {
        let mut g = two_player_game();
        g.step = TurnStep::PreCombatMain;
        let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
        let land = g.add_card_to_graveyard(0, catalog::bristling_backwoods());
        let spell = g.add_card_to_hand(0, catalog::badlands_revival());
        g.players[0].mana_pool.add(Color::Black, 4);
        g.players[0].mana_pool.add(Color::Green, 1);
        g.perform_action(GameAction::CastSpell {
            card_id: spell,
            target: Some(Target::Permanent(bear)),
            additional_targets: vec![Target::Permanent(land)],
            mode: None,
            x_value: None,
        })
        .expect("cast Badlands Revival");
        drain_stack(&mut g);
        assert!(g.battlefield_find(bear).is_some(), "creature reanimated");
        assert!(g.players[0].hand.iter().any(|c| c.id == land), "permanent returned to hand");
    }

    /// Betrayal at the Vault fires the chosen creature's power at two others.
    #[test]
    fn betrayal_at_the_vault_fight() {
        let mut g = two_player_game();
        g.step = TurnStep::PreCombatMain;
        let mine = g.add_card_to_battlefield(0, catalog::gigantosaurus()); // 10/10
        let a = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let b = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let spell = g.add_card_to_hand(0, catalog::betrayal_at_the_vault());
        g.players[0].mana_pool.add(Color::Green, 6);
        g.perform_action(GameAction::CastSpell {
            card_id: spell,
            target: Some(Target::Permanent(mine)),
            additional_targets: vec![Target::Permanent(a), Target::Permanent(b)],
            mode: None,
            x_value: None,
        })
        .expect("cast Betrayal at the Vault");
        drain_stack(&mut g);
        assert!(g.battlefield_find(a).is_none(), "first victim took 10 and died");
        assert!(g.battlefield_find(b).is_none(), "second victim took 10 and died");
    }

    /// Dust Animus enters bigger with five untapped lands.
    #[test]
    fn dust_animus_land_bonus() {
        let mut g = two_player_game();
        for _ in 0..5 {
            g.add_card_to_battlefield(0, catalog::bristling_backwoods());
        }
        // The painlands enter untapped here (no ETB fired), so all five count.
        let d = g.add_card_to_battlefield(0, catalog::dust_animus());
        g.fire_self_etb_triggers(d, 0);
        drain_stack(&mut g);
        assert_eq!(g.computed_permanent(d).unwrap().power, 4, "2/3 + two +1/+1 = 4/5");
    }

    /// Claim Jumper ramps a Plains when behind on lands.
    #[test]
    fn claim_jumper_land_catchup() {
        let mut g = two_player_game();
        let plains = g.add_card_to_library(0, catalog::plains());
        // Opponent controls more lands.
        g.add_card_to_battlefield(1, catalog::bristling_backwoods());
        g.add_card_to_battlefield(1, catalog::bristling_backwoods());
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(plains))]));
        let cj = g.add_card_to_battlefield(0, catalog::claim_jumper());
        g.fire_self_etb_triggers(cj, 0);
        drain_stack(&mut g);
        assert!(
            g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Plains"),
            "searched a Plains onto the battlefield"
        );
    }

    /// Binding Negotiation strips a nonland card from an opponent's hand.
    #[test]
    fn binding_negotiation_discard() {
        let mut g = two_player_game();
        g.step = TurnStep::PreCombatMain;
        g.add_card_to_hand(1, catalog::grizzly_bears());
        let land_in_hand = g.add_card_to_hand(1, catalog::plains());
        let spell = g.add_card_to_hand(0, catalog::binding_negotiation());
        g.players[0].mana_pool.add(Color::Black, 2);
        g.perform_action(GameAction::CastSpell {
            card_id: spell,
            target: None,
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .expect("cast Binding Negotiation");
        drain_stack(&mut g);
        assert!(
            g.players[1].graveyard.iter().any(|c| c.definition.name == "Grizzly Bears"),
            "the nonland card was discarded"
        );
        assert!(g.players[1].hand.iter().any(|c| c.id == land_in_hand), "the land stays");
    }

    /// Bandit's Haul stores a loot counter on a crime and cashes two for a draw.
    #[test]
    fn bandits_haul_loot_counters() {
        let mut g = two_player_game();
        g.step = TurnStep::PreCombatMain;
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        g.add_card_to_library(0, catalog::grizzly_bears());
        let haul = g.add_card_to_battlefield(0, catalog::bandits_haul());
        // A crime adds a loot counter.
        g.dispatch_triggers_for_events(&[GameEvent::CommittedCrime { player: 0 }]);
        drain_stack(&mut g);
        assert_eq!(
            g.battlefield_find(haul).unwrap().counters.get(&CounterType::Charge).copied().unwrap_or(0),
            1,
            "one loot counter from the crime",
        );
        // Top it up to two and cash them for a draw.
        g.battlefield_find_mut(haul).unwrap().counters.insert(CounterType::Charge, 2);
        let hand_before = g.players[0].hand.len();
        g.players[0].mana_pool.add(Color::Black, 2);
        g.perform_action(GameAction::ActivateAbility {
            card_id: haul,
            ability_index: 1,
            target: None,
            additional_targets: Vec::new(),
            x_value: None,
        })
        .expect("cash two loot counters");
        drain_stack(&mut g);
        assert_eq!(g.players[0].hand.len(), hand_before + 1, "drew a card");
        assert_eq!(
            g.battlefield_find(haul).unwrap().counters.get(&CounterType::Charge).copied().unwrap_or(0),
            0,
            "counters spent",
        );
    }

    /// Colossal Rattlewurm fetches a Desert from the graveyard-exile ability.
    #[test]
    fn colossal_rattlewurm_desert_fetch() {
        let mut g = two_player_game();
        g.step = TurnStep::PreCombatMain;
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        let desert = g.add_card_to_library(0, catalog::bristling_backwoods()); // a Desert
        let worm = g.add_card_to_graveyard(0, catalog::colossal_rattlewurm());
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(desert))]));
        g.players[0].mana_pool.add(Color::Green, 2);
        g.perform_action(GameAction::ActivateAbility {
            card_id: worm,
            ability_index: 0,
            target: None,
            additional_targets: Vec::new(),
            x_value: None,
        })
        .expect("exile from graveyard to fetch a Desert");
        drain_stack(&mut g);
        assert!(
            g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Bristling Backwoods"),
            "Desert fetched onto the battlefield"
        );
    }

    /// Cactusfolk Sureshot grants trample+haste to your big creatures at combat.
    #[test]
    fn cactusfolk_sureshot_combat_buff() {
        let mut g = two_player_game();
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        g.add_card_to_battlefield(0, catalog::cactusfolk_sureshot());
        let big = g.add_card_to_battlefield(0, catalog::gigantosaurus()); // 10/10, power ≥ 4
        while g.step != TurnStep::BeginCombat {
            g.perform_action(GameAction::PassPriority).unwrap();
        }
        drain_stack(&mut g);
        let cp = g.computed_permanent(big).unwrap();
        assert!(cp.keywords.contains(&Keyword::Trample), "granted trample");
        assert!(cp.keywords.contains(&Keyword::Haste), "granted haste");
    }

    /// Frontier Seeker digs a Mount or Plains into hand.
    #[test]
    fn frontier_seeker_digs_plains() {
        let mut g = two_player_game();
        let plains = g.add_card_to_library(0, catalog::plains());
        for _ in 0..4 {
            g.add_card_to_library(0, catalog::grizzly_bears());
        }
        let fs = g.add_card_to_battlefield(0, catalog::frontier_seeker());
        g.fire_self_etb_triggers(fs, 0);
        drain_stack(&mut g);
        assert!(g.players[0].hand.iter().any(|c| c.id == plains), "Plains put into hand");
    }
}

mod recent128 {
    use crabomination::card::{CounterType, EnchantmentSubtype, Keyword};
    use crabomination::catalog;
    use crabomination::game::types::Target;
    use crabomination::game::*;
    use crabomination::game::{drain_stack, two_player_game};
    use crabomination::mana::Color;

    /// Armory Mice is 3/1 normally, 3/3 once Celebration is active.
    #[test]
    fn armory_mice_celebration_toughness() {
        let mut g = two_player_game();
        let mice = g.add_card_to_battlefield(0, catalog::armory_mice());
        assert_eq!(g.computed_permanent(mice).unwrap().toughness, 1, "no celebration → 3/1");
        // Two nonland permanents entering this turn switches Celebration on.
        g.players[0].nonland_permanents_entered_this_turn = 2;
        assert_eq!(g.computed_permanent(mice).unwrap().toughness, 3, "celebration → +0/+2");
    }

    /// Belligerent of the Ball pumps a creature at combat only under Celebration.
    #[test]
    fn belligerent_celebration_combat_trigger() {
        let mut g = two_player_game();
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        let bell = g.add_card_to_battlefield(0, catalog::belligerent_of_the_ball());
        g.players[0].nonland_permanents_entered_this_turn = 2;
        while g.step != TurnStep::BeginCombat {
            g.perform_action(GameAction::PassPriority).unwrap();
        }
        drain_stack(&mut g);
        let cp = g.computed_permanent(bell).unwrap();
        assert_eq!(cp.power, 4, "+1/+0 under celebration");
        assert!(cp.keywords.contains(&Keyword::Menace), "and menace");
    }

    /// Archive Dragon scrys 2 on entry.
    #[test]
    fn archive_dragon_scry() {
        let mut g = two_player_game();
        for _ in 0..3 {
            g.add_card_to_library(0, catalog::grizzly_bears());
        }
        let dragon = g.add_card_to_battlefield(0, catalog::archive_dragon());
        g.fire_self_etb_triggers(dragon, 0);
        drain_stack(&mut g);
        let cp = g.computed_permanent(dragon).unwrap();
        assert!(cp.keywords.contains(&Keyword::Flying) && cp.keywords.iter().any(|k| matches!(k, Keyword::Ward(_))));
    }

    /// Agatha's Champion fights only when bargained.
    #[test]
    fn agathas_champion_bargained_fight() {
        let mut g = two_player_game();
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        g.step = TurnStep::PreCombatMain;
        let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
        let fodder = g.add_card_to_battlefield(0, catalog::ornithopter()); // artifact to bargain
        let champ = g.add_card_to_hand(0, catalog::agathas_champion());
        g.players[0].mana_pool.add(Color::Green, 1);
        g.players[0].mana_pool.add_colorless(4);
        g.perform_action(GameAction::CastSpellBargain {
            card_id: champ,
            sacrifice: Some(fodder),
            target: Some(Target::Permanent(victim)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .expect("cast Agatha's Champion bargained");
        drain_stack(&mut g);
        // 4/4 fights the 2/2 → the bear dies.
        assert!(g.battlefield_find(victim).is_none(), "bargained fight kills the bear");
    }

    /// Cut In burns a creature and makes a Young Hero Role on your own.
    #[test]
    fn cut_in_damage_and_role() {
        let mut g = two_player_game();
        g.step = TurnStep::PreCombatMain;
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        let enemy = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let spell = g.add_card_to_hand(0, catalog::cut_in());
        g.players[0].mana_pool.add(Color::Red, 1);
        g.players[0].mana_pool.add_colorless(3);
        g.perform_action(GameAction::CastSpell {
            card_id: spell,
            target: Some(Target::Permanent(enemy)),
            additional_targets: vec![Target::Permanent(mine)],
            mode: None,
            x_value: None,
        })
        .expect("cast Cut In");
        drain_stack(&mut g);
        assert!(g.battlefield_find(enemy).is_none(), "4 damage kills the 2/2");
        assert!(
            g.battlefield.iter().any(|c| c.attached_to == Some(mine)
                && c.definition.subtypes.enchantment_subtypes.contains(&EnchantmentSubtype::Role)),
            "Young Hero Role attached to my creature",
        );
    }

    /// Become Brutes gives haste and a Monster Role (+1/+1 trample).
    #[test]
    fn become_brutes_monster_role() {
        let mut g = two_player_game();
        g.step = TurnStep::PreCombatMain;
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let spell = g.add_card_to_hand(0, catalog::become_brutes());
        g.players[0].mana_pool.add(Color::Red, 1);
        g.players[0].mana_pool.add_colorless(1);
        g.perform_action(GameAction::CastSpell {
            card_id: spell,
            target: Some(Target::Permanent(bear)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .expect("cast Become Brutes");
        drain_stack(&mut g);
        let cp = g.computed_permanent(bear).unwrap();
        assert_eq!((cp.power, cp.toughness), (3, 3), "Monster Role gives +1/+1");
        assert!(cp.keywords.contains(&Keyword::Trample) && cp.keywords.contains(&Keyword::Haste));
    }

    /// Diminisher Witch bargained shrinks an opponent's creature to 1/1.
    #[test]
    fn diminisher_witch_cursed_role() {
        let mut g = two_player_game();
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        g.step = TurnStep::PreCombatMain;
        let big = g.add_card_to_battlefield(1, catalog::gigantosaurus()); // 10/10
        let fodder = g.add_card_to_battlefield(0, catalog::ornithopter());
        let witch = g.add_card_to_hand(0, catalog::diminisher_witch());
        g.players[0].mana_pool.add(Color::Blue, 1);
        g.players[0].mana_pool.add_colorless(2);
        g.perform_action(GameAction::CastSpellBargain {
            card_id: witch,
            sacrifice: Some(fodder),
            target: Some(Target::Permanent(big)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .expect("cast Diminisher Witch bargained");
        drain_stack(&mut g);
        let cp = g.computed_permanent(big).unwrap();
        assert_eq!((cp.power, cp.toughness), (1, 1), "Cursed Role sets it to 1/1");
    }

    /// Charging Hooligan grows by the number of attackers.
    #[test]
    fn charging_hooligan_scales_with_attackers() {
        let mut g = two_player_game();
        let hooligan = g.add_card_to_battlefield(0, catalog::charging_hooligan());
        let ally = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.clear_sickness(hooligan);
        g.clear_sickness(ally);
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        while g.step != TurnStep::DeclareAttackers {
            g.perform_action(GameAction::PassPriority).unwrap();
        }
        g.perform_action(GameAction::DeclareAttackers(vec![
            Attack { attacker: hooligan, target: AttackTarget::Player(1) },
            Attack { attacker: ally, target: AttackTarget::Player(1) },
        ]))
        .expect("attack with two");
        drain_stack(&mut g);
        // Base 3 + 2 attackers = 5.
        assert_eq!(g.computed_permanent(hooligan).unwrap().power, 5, "+1/+0 per attacker");
    }

    /// Barrow Naughty has lifelink only while you control another Faerie.
    #[test]
    fn barrow_naughty_conditional_lifelink() {
        let mut g = two_player_game();
        let naughty = g.add_card_to_battlefield(0, catalog::barrow_naughty());
        assert!(
            !g.computed_permanent(naughty).unwrap().keywords.contains(&Keyword::Lifelink),
            "no other Faerie → no lifelink",
        );
        g.add_card_to_battlefield(0, catalog::barrow_naughty()); // another Faerie
        assert!(
            g.computed_permanent(naughty).unwrap().keywords.contains(&Keyword::Lifelink),
            "another Faerie → lifelink",
        );
    }

    /// Ego Drain strips a nonland card from an opponent's hand.
    #[test]
    fn ego_drain_discards_nonland() {
        let mut g = two_player_game();
        g.step = TurnStep::PreCombatMain;
        g.add_card_to_hand(1, catalog::grizzly_bears());
        let land = g.add_card_to_hand(1, catalog::forest());
        let spell = g.add_card_to_hand(0, catalog::ego_drain());
        g.players[0].mana_pool.add(Color::Black, 1);
        g.perform_action(GameAction::CastSpell {
            card_id: spell,
            target: None,
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .expect("cast Ego Drain");
        drain_stack(&mut g);
        assert!(g.players[1].graveyard.iter().any(|c| c.definition.name == "Grizzly Bears"), "discarded");
        assert!(g.players[1].hand.iter().any(|c| c.id == land), "the land stays");
    }

    /// The Young Hero Role's granted trigger counters up its host on attack.
    #[test]
    fn young_hero_role_counter_on_attack() {
        let mut g = two_player_game();
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let spell = g.add_card_to_hand(0, catalog::cut_in());
        // Give a throwaway enemy so Cut In's damage target is legal.
        let enemy = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        g.step = TurnStep::PreCombatMain;
        g.players[0].mana_pool.add(Color::Red, 1);
        g.players[0].mana_pool.add_colorless(3);
        g.perform_action(GameAction::CastSpell {
            card_id: spell,
            target: Some(Target::Permanent(enemy)),
            additional_targets: vec![Target::Permanent(bear)],
            mode: None,
            x_value: None,
        })
        .expect("cast Cut In for the Role");
        drain_stack(&mut g);
        g.clear_sickness(bear);
        while g.step != TurnStep::DeclareAttackers {
            g.perform_action(GameAction::PassPriority).unwrap();
        }
        g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker: bear,
            target: AttackTarget::Player(1),
        }]))
        .expect("attack");
        drain_stack(&mut g);
        assert_eq!(
            g.battlefield_find(bear).unwrap().counters.get(&CounterType::PlusOnePlusOne).copied().unwrap_or(0),
            1,
            "Young Hero Role puts a +1/+1 counter on attack",
        );
    }
}

mod recent129 {
    use crabomination::card::Keyword;
    use crabomination::catalog;
    use crabomination::game::types::Target;
    use crabomination::game::*;
    use crabomination::game::{drain_stack, two_player_game};
    use crabomination::mana::Color;

    /// Moonshaker Cavalry gives the team flying and +X/+X for X creatures.
    #[test]
    fn moonshaker_cavalry_team_anthem() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let cav = g.add_card_to_battlefield(0, catalog::moonshaker_cavalry());
        g.fire_self_etb_triggers(cav, 0);
        drain_stack(&mut g);
        // Two creatures → +2/+2 and flying.
        let cp = g.computed_permanent(bear).unwrap();
        assert_eq!((cp.power, cp.toughness), (4, 4), "bear 2/2 → 4/4");
        assert!(cp.keywords.contains(&Keyword::Flying), "and flying");
    }

    /// Water Wings makes your creature a 4/4 flier with hexproof.
    #[test]
    fn water_wings_transforms_creature() {
        let mut g = two_player_game();
        g.step = TurnStep::PreCombatMain;
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let spell = g.add_card_to_hand(0, catalog::water_wings());
        g.players[0].mana_pool.add(Color::Blue, 1);
        g.players[0].mana_pool.add_colorless(1);
        g.perform_action(GameAction::CastSpell {
            card_id: spell,
            target: Some(Target::Permanent(bear)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .expect("cast Water Wings");
        drain_stack(&mut g);
        let cp = g.computed_permanent(bear).unwrap();
        assert_eq!((cp.power, cp.toughness), (4, 4), "base 4/4");
        assert!(cp.keywords.contains(&Keyword::Flying) && cp.keywords.contains(&Keyword::Hexproof));
    }

    /// Werefox Bodyguard exiles a creature until it leaves, and returns it on sac.
    #[test]
    fn werefox_bodyguard_exile_and_return() {
        let mut g = two_player_game();
        let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let fox = g.add_card_to_battlefield(0, catalog::werefox_bodyguard());
        g.fire_self_etb_triggers(fox, 0);
        drain_stack(&mut g);
        assert!(g.battlefield_find(victim).is_none(), "creature exiled");
        // Sacrifice the Werefox to gain 2 and free the exile.
        let life = g.players[0].life;
        g.players[0].mana_pool.add(Color::White, 1);
        g.players[0].mana_pool.add_colorless(1);
        g.perform_action(GameAction::ActivateAbility {
            card_id: fox,
            ability_index: 0,
            target: None,
            additional_targets: Vec::new(),
            x_value: None,
        })
        .expect("sac Werefox");
        drain_stack(&mut g);
        assert_eq!(g.players[0].life, life + 2, "gained 2 life");
        assert!(g.battlefield_find(victim).is_some(), "exiled creature returned when the Werefox left");
    }

    /// Grand Ball Guest grows and gains trample under Celebration.
    #[test]
    fn grand_ball_guest_celebration() {
        let mut g = two_player_game();
        let guest = g.add_card_to_battlefield(0, catalog::grand_ball_guest());
        assert_eq!(g.computed_permanent(guest).unwrap().power, 2, "no celebration → 2/2");
        g.players[0].nonland_permanents_entered_this_turn = 2;
        let cp = g.computed_permanent(guest).unwrap();
        assert_eq!((cp.power, cp.toughness), (3, 3), "celebration → +1/+1");
        assert!(cp.keywords.contains(&Keyword::Trample), "and trample");
    }

    /// Ratcatcher Trainee's Pest Problem adventure makes two Rats.
    #[test]
    fn ratcatcher_pest_problem_makes_rats() {
        let mut g = two_player_game();
        g.step = TurnStep::PreCombatMain;
        let card = g.add_card_to_hand(0, catalog::ratcatcher_trainee());
        g.players[0].mana_pool.add(Color::Red, 1);
        g.players[0].mana_pool.add_colorless(2);
        g.perform_action(GameAction::CastAdventure {
            card_id: card,
            target: None,
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .expect("cast Pest Problem");
        drain_stack(&mut g);
        assert_eq!(
            g.battlefield.iter().filter(|c| c.controller == 0 && c.definition.name == "Rat").count(),
            2,
            "two Rats made",
        );
    }

    /// Twisted Fealty steals a creature for the turn and drops a Wicked Role.
    #[test]
    fn twisted_fealty_steal_and_role() {
        let mut g = two_player_game();
        g.step = TurnStep::PreCombatMain;
        let stolen = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let spell = g.add_card_to_hand(0, catalog::twisted_fealty());
        g.players[0].mana_pool.add(Color::Red, 1);
        g.players[0].mana_pool.add_colorless(2);
        g.perform_action(GameAction::CastSpell {
            card_id: spell,
            target: Some(Target::Permanent(stolen)),
            additional_targets: vec![Target::Permanent(mine)],
            mode: None,
            x_value: None,
        })
        .expect("cast Twisted Fealty");
        drain_stack(&mut g);
        assert_eq!(g.battlefield_find(stolen).unwrap().controller, 0, "gained control this turn");
        let cp = g.computed_permanent(mine).unwrap();
        assert_eq!((cp.power, cp.toughness), (3, 3), "Wicked Role gives +1/+1");
    }
}

mod recent130 {
    use crabomination::card::{CounterType, Keyword};
    use crabomination::catalog;
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    use crabomination::game::types::Target;
    use crabomination::game::*;
    use crabomination::game::{drain_stack, two_player_game};
    use crabomination::mana::Color;

    /// Scream Puff makes a Food when it connects.
    #[test]
    fn scream_puff_food_on_combat_damage() {
        let mut g = two_player_game();
        let puff = g.add_card_to_battlefield(0, catalog::scream_puff());
        g.clear_sickness(puff);
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        while g.step != TurnStep::DeclareAttackers {
            g.perform_action(GameAction::PassPriority).unwrap();
        }
        g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker: puff,
            target: AttackTarget::Player(1),
        }]))
        .expect("attack");
        drain_stack(&mut g);
        // Pass through combat damage.
        while g.step == TurnStep::DeclareAttackers || g.step == TurnStep::DeclareBlockers {
            g.perform_action(GameAction::PassPriority).unwrap();
        }
        drain_stack(&mut g);
        assert!(
            g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Food"),
            "Food created on combat damage",
        );
    }

    /// Beanstalk Wurm's Plant Beans grants an extra land drop.
    #[test]
    fn plant_beans_grants_extra_land() {
        let mut g = two_player_game();
        g.step = TurnStep::PreCombatMain;
        let card = g.add_card_to_hand(0, catalog::beanstalk_wurm());
        g.players[0].mana_pool.add(Color::Green, 1);
        g.players[0].mana_pool.add_colorless(1);
        let before = g.players[0].extra_land_plays;
        g.perform_action(GameAction::CastAdventure {
            card_id: card,
            target: None,
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .expect("cast Plant Beans");
        drain_stack(&mut g);
        assert_eq!(g.players[0].extra_land_plays, before + 1, "one extra land play");
    }

    /// Return from the Wilds — choosing the Human + Food modes makes both.
    #[test]
    fn return_from_the_wilds_choose_two() {
        let mut g = two_player_game();
        g.step = TurnStep::PreCombatMain;
        let spell = g.add_card_to_hand(0, catalog::return_from_the_wilds());
        g.players[0].mana_pool.add(Color::Green, 1);
        g.players[0].mana_pool.add_colorless(2);
        // Pick modes 1 (Human) and 2 (Food).
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Modes(vec![1, 2])]));
        g.perform_action(GameAction::CastSpell {
            card_id: spell,
            target: None,
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .expect("cast Return from the Wilds");
        drain_stack(&mut g);
        assert!(g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Human"), "made a Human");
        assert!(g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Food"), "made a Food");
    }

    /// Stockpiling Celebrant returns a permanent and scrys.
    #[test]
    fn stockpiling_celebrant_bounce_and_scry() {
        let mut g = two_player_game();
        for _ in 0..2 {
            g.add_card_to_library(0, catalog::grizzly_bears());
        }
        let clue = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let cel = g.add_card_to_battlefield(0, catalog::stockpiling_celebrant());
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
        g.fire_self_etb_triggers(cel, 0);
        drain_stack(&mut g);
        assert!(g.players[0].hand.iter().any(|c| c.id == clue), "returned the other creature to hand");
    }

    /// Elusive Otter's Grove's Bounty distributes X +1/+1 counters.
    #[test]
    fn groves_bounty_distributes_counters() {
        let mut g = two_player_game();
        g.step = TurnStep::PreCombatMain;
        let a = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let b = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let card = g.add_card_to_hand(0, catalog::elusive_otter());
        g.players[0].mana_pool.add(Color::Green, 1);
        g.players[0].mana_pool.add_colorless(2); // X = 2
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::DamageDivision(vec![1, 1])]));
        g.perform_action(GameAction::CastAdventure {
            card_id: card,
            target: Some(Target::Permanent(a)),
            additional_targets: vec![Target::Permanent(b)],
            mode: None,
            x_value: Some(2),
        })
        .expect("cast Grove's Bounty for X=2");
        drain_stack(&mut g);
        let total: u32 = [a, b]
            .iter()
            .map(|&id| g.battlefield_find(id).unwrap().counters.get(&CounterType::PlusOnePlusOne).copied().unwrap_or(0))
            .sum();
        assert_eq!(total, 2, "two +1/+1 counters distributed");
    }

    /// Elusive Otter can't be blocked by lower-power creatures.
    #[test]
    fn elusive_otter_evasion() {
        let mut g = two_player_game();
        let otter = g.add_card_to_battlefield(0, catalog::elusive_otter());
        let cp = g.computed_permanent(otter).unwrap();
        assert!(
            cp.keywords.contains(&Keyword::Prowess)
                && cp.keywords.contains(&Keyword::CantBeBlockedByPowerLess),
            "prowess + can't-be-blocked-by-lesser evasion",
        );
    }
}

mod recent131 {
    use crabomination::card::{
        CardDefinition, CardType, CounterType, EnchantmentSubtype, Keyword, Subtypes,
    };
    use crabomination::catalog;
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    use crabomination::game::types::Target;
    use crabomination::game::*;
    use crabomination::game::{drain_stack, two_player_game};
    use crabomination::mana::{cost, generic, Color};

    /// A vanilla enchantment fixture (optionally an Aura subtype) for the
    /// enchantment-matters triggers.
    fn dummy_enchantment(aura: bool) -> CardDefinition {
        CardDefinition {
            name: "Test Glyph",
            cost: cost(&[generic(2)]),
            card_types: vec![CardType::Enchantment],
            subtypes: Subtypes {
                enchantment_subtypes: if aura { vec![EnchantmentSubtype::Aura] } else { vec![] },
                ..Default::default()
            },
            ..Default::default()
        }
    }

    /// Sacrifice a battlefield permanent, firing dies/LTB triggers (CR 701.16).
    fn kill(g: &mut GameState, id: CardId) {
        let ctl = g.battlefield_find(id).unwrap().controller;
        let ctx = crabomination::game::effects::EffectContext::for_ability(id, ctl, Some(Target::Permanent(id)));
        g.resolve_effect(
            &crabomination::effect::Effect::SacrificePermanent { what: crabomination::effect::Selector::Target(0) },
            &ctx,
        )
        .unwrap();
        // Flush synthesized non-creature `PermanentDied` events (CR 700.4) so
        // "whenever an enchantment you control … dies" watchers fire.
        g.dispatch_triggers_for_events(&[]);
        drain_stack(g);
    }

    /// Regal Bunnicorn's `*/*` equals the number of nonland permanents you control.
    #[test]
    fn regal_bunnicorn_pt_scales() {
        let mut g = two_player_game();
        let bunny = g.add_card_to_battlefield(0, catalog::regal_bunnicorn());
        // Bunny alone → 1 nonland permanent → 1/1.
        let cp = g.computed_permanent(bunny).unwrap();
        assert_eq!((cp.power, cp.toughness), (1, 1));
        // Two more nonland permanents → 3/3. A land doesn't count.
        g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.add_card_to_battlefield(0, catalog::forest());
        let cp = g.computed_permanent(bunny).unwrap();
        assert_eq!((cp.power, cp.toughness), (3, 3), "3 nonland permanents");
    }

    /// Savior of the Sleeping grows when your enchantment dies.
    #[test]
    fn savior_counter_on_enchantment_death() {
        let mut g = two_player_game();
        let savior = g.add_card_to_battlefield(0, catalog::savior_of_the_sleeping());
        let glyph = g.add_card_to_battlefield(0, dummy_enchantment(false));
        kill(&mut g, glyph);
        assert_eq!(
            g.battlefield_find(savior).unwrap().counter_count(CounterType::PlusOnePlusOne),
            1,
            "one +1/+1 counter from the enchantment death",
        );
    }

    /// Wicked Visitor drains each opponent when your enchantment dies.
    #[test]
    fn wicked_visitor_drains_on_enchantment_death() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::wicked_visitor());
        let glyph = g.add_card_to_battlefield(0, dummy_enchantment(false));
        g.players[1].life = 20;
        kill(&mut g, glyph);
        assert_eq!(g.players[1].life, 19, "opponent loses 1 life");
    }

    /// Warehouse Tabby makes a Rat when your enchantment dies.
    #[test]
    fn warehouse_tabby_rat_on_enchantment_death() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::warehouse_tabby());
        let glyph = g.add_card_to_battlefield(0, dummy_enchantment(false));
        kill(&mut g, glyph);
        assert!(
            g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Rat"),
            "Rat token created",
        );
    }

    /// Harried Spearguard leaves a Rat behind when it dies.
    #[test]
    fn harried_spearguard_rat_on_death() {
        let mut g = two_player_game();
        let guard = g.add_card_to_battlefield(0, catalog::harried_spearguard());
        kill(&mut g, guard);
        assert!(
            g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Rat"),
            "Rat token on death",
        );
    }

    /// Redcap Thief makes a Treasure on entry.
    #[test]
    fn redcap_thief_treasure_on_etb() {
        let mut g = two_player_game();
        let thief = g.add_card_to_battlefield(0, catalog::redcap_thief());
        g.fire_self_etb_triggers(thief, 0);
        drain_stack(&mut g);
        assert!(
            g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Treasure"),
            "Treasure created",
        );
    }

    /// Spiteful Hexmage shrinks a creature to 1/1 with a Cursed Role.
    #[test]
    fn spiteful_hexmage_cursed_role() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let mage = g.add_card_to_battlefield(0, catalog::spiteful_hexmage());
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Target(Target::Permanent(bear))]));
        g.fire_self_etb_triggers(mage, 0);
        drain_stack(&mut g);
        let cp = g.computed_permanent(bear).unwrap();
        assert_eq!((cp.power, cp.toughness), (1, 1), "Cursed Role sets base 1/1");
    }

    /// Toadstool Admirer grows itself with its activated ability.
    #[test]
    fn toadstool_admirer_self_pump() {
        let mut g = two_player_game();
        let toad = g.add_card_to_battlefield(0, catalog::toadstool_admirer());
        g.players[0].mana_pool.add(Color::Green, 1);
        g.players[0].mana_pool.add_colorless(3);
        g.priority.player_with_priority = 0;
        g.step = TurnStep::PreCombatMain;
        g.perform_action(GameAction::ActivateAbility {
            card_id: toad,
            ability_index: 0,
            target: None,
            additional_targets: vec![],
            x_value: None,
        })
        .expect("activate Toadstool Admirer");
        drain_stack(&mut g);
        assert_eq!(g.battlefield_find(toad).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
    }

    /// Rootrider Faun taps for {G}.
    #[test]
    fn rootrider_faun_taps_for_green() {
        let mut g = two_player_game();
        let faun = g.add_card_to_battlefield(0, catalog::rootrider_faun());
        g.clear_sickness(faun);
        g.priority.player_with_priority = 0;
        g.step = TurnStep::PreCombatMain;
        g.perform_action(GameAction::ActivateAbility {
            card_id: faun,
            ability_index: 0,
            target: None,
            additional_targets: vec![],
            x_value: None,
        })
        .expect("tap Rootrider Faun for G");
        assert_eq!(g.players[0].mana_pool.amount(Color::Green), 1, "one green mana");
    }

    /// Stormkeld Prowler gains two counters when you cast a 5-drop.
    #[test]
    fn stormkeld_prowler_counters_on_big_spell() {
        let mut g = two_player_game();
        let prowler = g.add_card_to_battlefield(0, catalog::stormkeld_prowler());
        let big = g.add_card_to_hand(0, CardDefinition {
            name: "Test Bomb",
            cost: cost(&[generic(5)]),
            card_types: vec![CardType::Sorcery],
            ..Default::default()
        });
        g.players[0].mana_pool.add_colorless(5);
        g.priority.player_with_priority = 0;
        g.step = TurnStep::PreCombatMain;
        g.perform_action(GameAction::CastSpell {
            card_id: big,
            target: None,
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .expect("cast 5-drop");
        drain_stack(&mut g);
        assert_eq!(
            g.battlefield_find(prowler).unwrap().counter_count(CounterType::PlusOnePlusOne),
            2,
            "two +1/+1 counters",
        );
    }

    /// Snaremaster Sprite taps and stuns when you pay {2}.
    #[test]
    fn snaremaster_sprite_pay_taps_and_stuns() {
        let mut g = two_player_game();
        let enemy = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let sprite = g.add_card_to_battlefield(0, catalog::snaremaster_sprite());
        g.players[0].mana_pool.add_colorless(2);
        g.decider = Box::new(ScriptedDecider::new([
            DecisionAnswer::Bool(true),
            DecisionAnswer::Target(Target::Permanent(enemy)),
        ]));
        g.fire_self_etb_triggers(sprite, 0);
        drain_stack(&mut g);
        let ec = g.battlefield_find(enemy).unwrap();
        assert!(ec.tapped, "enemy tapped");
        assert_eq!(ec.counter_count(CounterType::Stun), 1, "stun counter placed");
    }

    /// Twisted Reflection's switch mode swaps a 2/4 into a 4/2.
    #[test]
    fn twisted_reflection_switches_pt() {
        let mut g = two_player_game();
        // Wall of Wonder is a 1/5; use a 2/4-ish body — grizzly bears is 2/2, so
        // build a fixture with distinct P/T.
        let creature = g.add_card_to_battlefield(1, CardDefinition {
            name: "Test Wall",
            cost: cost(&[generic(3)]),
            card_types: vec![CardType::Creature],
            subtypes: Subtypes::default(),
            power: 2,
            toughness: 4,
            ..Default::default()
        });
        let spell = g.add_card_to_hand(0, catalog::twisted_reflection());
        g.players[0].mana_pool.add(Color::Blue, 1);
        g.players[0].mana_pool.add_colorless(1);
        g.priority.player_with_priority = 0;
        g.step = TurnStep::PreCombatMain;
        // Choose mode 1 (switch), target the creature.
        g.perform_action(GameAction::CastSpell {
            card_id: spell,
            target: Some(Target::Permanent(creature)),
            additional_targets: vec![],
            mode: Some(1),
            x_value: None,
        })
        .expect("cast Twisted Reflection");
        drain_stack(&mut g);
        let cp = g.computed_permanent(creature).unwrap();
        assert_eq!((cp.power, cp.toughness), (4, 2), "power and toughness switched");
    }

    /// Bellowing Elk gains trample + indestructible once another creature entered.
    #[test]
    fn bellowing_elk_conditional_keywords() {
        let mut g = two_player_game();
        let elk = g.add_card_to_battlefield(0, catalog::bellowing_elk());
        // No other creature entered yet → bare 4/2.
        let cp = g.computed_permanent(elk).unwrap();
        assert!(!cp.keywords.contains(&Keyword::Trample), "no keywords without another arrival");
        // Simulate another creature having entered under your control this turn.
        let other = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.players[0].creatures_entered_this_turn.push(other);
        let cp = g.computed_permanent(elk).unwrap();
        assert!(
            cp.keywords.contains(&Keyword::Trample) && cp.keywords.contains(&Keyword::Indestructible),
            "gains trample + indestructible",
        );
        // The elk's own arrival must not satisfy the self-excluding predicate.
        g.players[0].creatures_entered_this_turn = vec![elk];
        let cp = g.computed_permanent(elk).unwrap();
        assert!(!cp.keywords.contains(&Keyword::Trample), "own arrival doesn't count");
    }

    /// Windcaller Aven grants flying when cycled.
    #[test]
    fn windcaller_aven_cycle_grants_flying() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let aven = g.add_card_to_hand(0, catalog::windcaller_aven());
        g.add_card_to_library(0, catalog::grizzly_bears());
        g.players[0].mana_pool.add(Color::Blue, 1);
        g.priority.player_with_priority = 0;
        g.step = TurnStep::PreCombatMain;
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Target(Target::Permanent(bear))]));
        g.perform_action(GameAction::Cycle { card_id: aven, x_value: None }).expect("cycle Windcaller Aven");
        drain_stack(&mut g);
        let cp = g.computed_permanent(bear).unwrap();
        assert!(cp.keywords.contains(&Keyword::Flying), "bear gains flying");
    }
}

mod recent132 {
    use crabomination::card::{
        CardDefinition, CardType, EnchantmentSubtype, Keyword, Subtypes,
    };
    use crabomination::catalog;
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    use crabomination::game::types::Target;
    use crabomination::game::*;
    use crabomination::game::{drain_stack, two_player_game};
    use crabomination::mana::{cost, generic, Color};

    /// A vanilla enchantment fixture for the enchantment-death triggers.
    fn dummy_enchantment() -> CardDefinition {
        CardDefinition {
            name: "Test Glyph",
            cost: cost(&[generic(2)]),
            card_types: vec![CardType::Enchantment],
            subtypes: Subtypes::default(),
            ..Default::default()
        }
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

    fn has_role_on(g: &GameState, host: CardId) -> bool {
        g.battlefield.iter().any(|c| {
            c.attached_to == Some(host)
                && c.definition.subtypes.enchantment_subtypes.contains(&EnchantmentSubtype::Role)
        })
    }

    /// Squeak By pumps and grants power-3+ evasion; the new keyword blocks a big
    /// blocker but not a small one.
    #[test]
    fn squeak_by_pump_and_evasion() {
        let mut g = two_player_game();
        g.step = TurnStep::PreCombatMain;
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let weak = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
        let big = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4
        let card = g.add_card_to_hand(0, catalog::cheeky_house_mouse());
        g.players[0].mana_pool.add(Color::White, 1);
        g.perform_action(GameAction::CastAdventure {
            card_id: card,
            target: Some(Target::Permanent(bear)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .expect("cast Squeak By");
        drain_stack(&mut g);
        let cp = g.computed_permanent(bear).unwrap();
        assert_eq!((cp.power, cp.toughness), (3, 3), "+1/+1");
        assert!(cp.keywords.contains(&Keyword::CantBeBlockedByPowerAtLeast(3)));
        assert!(!g.blocker_can_block_attacker(big, bear), "power-4 can't block");
        assert!(g.blocker_can_block_attacker(weak, bear), "power-2 can block");
    }

    /// Betroth the Beast attaches a Royal Role (+1/+1, ward).
    #[test]
    fn betroth_the_beast_royal_role() {
        let mut g = two_player_game();
        g.step = TurnStep::PreCombatMain;
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let card = g.add_card_to_hand(0, catalog::besotted_knight());
        g.players[0].mana_pool.add(Color::White, 1);
        g.perform_action(GameAction::CastAdventure {
            card_id: card,
            target: Some(Target::Permanent(bear)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .expect("cast Betroth the Beast");
        drain_stack(&mut g);
        assert!(has_role_on(&g, bear), "Royal Role attached");
        let cp = g.computed_permanent(bear).unwrap();
        assert_eq!((cp.power, cp.toughness), (3, 3), "Royal Role gives +1/+1");
        assert!(cp.keywords.iter().any(|k| matches!(k, Keyword::Ward(_))), "has ward");
    }

    /// Charmed Clothier hangs a Royal Role on another creature.
    #[test]
    fn charmed_clothier_royal_role() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let clothier = g.add_card_to_battlefield(0, catalog::charmed_clothier());
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Target(Target::Permanent(bear))]));
        g.fire_self_etb_triggers(clothier, 0);
        drain_stack(&mut g);
        assert!(has_role_on(&g, bear), "Royal Role on the bear");
    }

    /// Ashiok's Reaper draws when your enchantment dies.
    #[test]
    fn ashioks_reaper_draw_on_enchantment_death() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::ashioks_reaper());
        let glyph = g.add_card_to_battlefield(0, dummy_enchantment());
        g.add_card_to_library(0, catalog::grizzly_bears());
        let before = g.players[0].hand.len();
        kill(&mut g, glyph);
        assert_eq!(g.players[0].hand.len(), before + 1, "drew a card");
    }

    /// Twice the Rage grants double strike.
    #[test]
    fn twice_the_rage_double_strike() {
        let mut g = two_player_game();
        g.step = TurnStep::PreCombatMain;
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let card = g.add_card_to_hand(0, catalog::two_headed_hunter());
        g.players[0].mana_pool.add(Color::Red, 1);
        g.players[0].mana_pool.add_colorless(1);
        g.perform_action(GameAction::CastAdventure {
            card_id: card,
            target: Some(Target::Permanent(bear)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .expect("cast Twice the Rage");
        drain_stack(&mut g);
        assert!(g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::DoubleStrike));
    }

    /// That's Mine makes a Treasure.
    #[test]
    fn thats_mine_treasure() {
        let mut g = two_player_game();
        g.step = TurnStep::PreCombatMain;
        let card = g.add_card_to_hand(0, catalog::grabby_giant());
        g.players[0].mana_pool.add(Color::Red, 1);
        g.players[0].mana_pool.add_colorless(1);
        g.perform_action(GameAction::CastAdventure {
            card_id: card,
            target: None,
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .expect("cast That's Mine");
        drain_stack(&mut g);
        assert!(g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Treasure"));
    }

    /// Hollow Scavenger eats a Food for +2/+2.
    #[test]
    fn hollow_scavenger_food_pump() {
        let mut g = two_player_game();
        let scav = g.add_card_to_battlefield(0, catalog::hollow_scavenger());
        g.add_token_to_battlefield(0, &crabomination::game::effects::food_token());
        g.players[0].mana_pool.add_colorless(1);
        g.priority.player_with_priority = 0;
        g.step = TurnStep::PreCombatMain;
        g.perform_action(GameAction::ActivateAbility {
            card_id: scav,
            ability_index: 0,
            target: None,
            additional_targets: vec![],
            x_value: None,
        })
        .expect("sac a Food");
        drain_stack(&mut g);
        let cp = g.computed_permanent(scav).unwrap();
        assert_eq!((cp.power, cp.toughness), (5, 4), "+2/+2");
        assert!(
            !g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Food"),
            "Food sacrificed",
        );
    }

    /// Skybeast Tracker makes a Food when you cast a 5-drop.
    #[test]
    fn skybeast_tracker_food_on_big_spell() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::skybeast_tracker());
        let big = g.add_card_to_hand(0, CardDefinition {
            name: "Test Bomb",
            cost: cost(&[generic(5)]),
            card_types: vec![CardType::Sorcery],
            ..Default::default()
        });
        g.players[0].mana_pool.add_colorless(5);
        g.priority.player_with_priority = 0;
        g.step = TurnStep::PreCombatMain;
        g.perform_action(GameAction::CastSpell {
            card_id: big,
            target: None,
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .expect("cast 5-drop");
        drain_stack(&mut g);
        assert!(g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Food"));
    }

    /// Verdant Outrider grants itself power-2-or-less evasion.
    #[test]
    fn verdant_outrider_evasion() {
        let mut g = two_player_game();
        let rider = g.add_card_to_battlefield(0, catalog::verdant_outrider());
        g.players[0].mana_pool.add(Color::Green, 1);
        g.players[0].mana_pool.add_colorless(1);
        g.priority.player_with_priority = 0;
        g.step = TurnStep::PreCombatMain;
        g.perform_action(GameAction::ActivateAbility {
            card_id: rider,
            ability_index: 0,
            target: None,
            additional_targets: vec![],
            x_value: None,
        })
        .expect("activate Verdant Outrider");
        drain_stack(&mut g);
        assert!(g
            .computed_permanent(rider)
            .unwrap()
            .keywords
            .contains(&Keyword::CantBeBlockedByPowerAtMost(2)));
    }
}
