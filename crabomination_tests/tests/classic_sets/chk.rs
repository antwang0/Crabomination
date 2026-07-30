//! Champions of Kamigawa — Spirit/Arcane bodies: Offering Patrons (tested in
//! `cr_rules`), Bushido, Soulshift, sacrifice/tap activated abilities.

use crabomination::game::*;
use crabomination::card::Keyword;
use crabomination::catalog;
use crabomination::game::two_player_game;

/// Activate an ability (optionally with an X value) and drain the stack.
macro_rules! act {
    ($g:ident, $id:expr, $idx:expr, $tgt:expr) => {{
        $g.perform_action(GameAction::ActivateAbility {
            card_id: $id, ability_index: $idx, target: $tgt,
            additional_targets: Vec::new(), x_value: None,
        }).expect("activate ability");
        drain_stack(&mut $g);
    }};
    ($g:ident, $id:expr, $idx:expr, $tgt:expr, x = $x:expr) => {{
        $g.perform_action(GameAction::ActivateAbility {
            card_id: $id, ability_index: $idx, target: $tgt,
            additional_targets: Vec::new(), x_value: Some($x),
        }).expect("activate ability");
        drain_stack(&mut $g);
    }};
}

// ── Printed stats / keyword table ────────────────────────────────────────────

/// Catalog-level printed P/T and keyword checks, one row per card.
#[test]
fn chk_printed_stats_and_keywords() {
    use crabomination::card::LandType;
    use crabomination::mana::Color;
    let rows = [
        ("Kitsune Blademaster", catalog::kitsune_blademaster(), None,
            vec![Keyword::FirstStrike, Keyword::Bushido(1)]),
        ("Kodama of the North Tree", catalog::kodama_of_the_north_tree(), Some((6, 4)),
            vec![Keyword::Trample, Keyword::Shroud]),
        ("Nezumi Cutthroat", catalog::nezumi_cutthroat(), None,
            vec![Keyword::Fear, Keyword::CantBlock]),
        ("Kami of Lunacy", catalog::kami_of_lunacy(), Some((4, 1)), vec![Keyword::Flying]),
        ("Venerable Kumo", catalog::venerable_kumo(), None, vec![Keyword::Reach]),
        ("Nagao, Bound by Honor", catalog::nagao_bound_by_honor(), None, vec![Keyword::Bushido(1)]),
        ("Kami of Old Stone", catalog::kami_of_old_stone(), Some((1, 7)), vec![]),
        ("Devoted Retainer", catalog::devoted_retainer(), None, vec![Keyword::Bushido(1)]),
        ("Ronin Houndmaster", catalog::ronin_houndmaster(), None,
            vec![Keyword::Haste, Keyword::Bushido(1)]),
        ("Mothrider Samurai", catalog::mothrider_samurai(), None,
            vec![Keyword::Flying, Keyword::Bushido(1)]),
        ("Sokenzan Bruiser", catalog::sokenzan_bruiser(), None,
            vec![Keyword::Landwalk(LandType::Mountain)]),
        ("Moss Kami", catalog::moss_kami(), None, vec![Keyword::Trample]),
        ("Numai Outcast", catalog::numai_outcast(), None, vec![Keyword::Bushido(2)]),
        ("Konda, Lord of Eiganjo", catalog::konda_lord_of_eiganjo(), None,
            vec![Keyword::Indestructible, Keyword::Vigilance, Keyword::Bushido(5)]),
        ("Vine Kami", catalog::vine_kami(), None, vec![Keyword::Menace]),
        ("Nezumi Ronin", catalog::nezumi_ronin(), None, vec![Keyword::Bushido(1)]),
        ("Villainous Ogre", catalog::villainous_ogre(), None, vec![Keyword::CantBlock]),
        ("Samurai Enforcers", catalog::samurai_enforcers(), Some((4, 4)), vec![Keyword::Bushido(2)]),
        ("Kami of the Palace Fields", catalog::kami_of_the_palace_fields(), None,
            vec![Keyword::Flying, Keyword::FirstStrike]),
        ("Ronin Cavekeeper", catalog::ronin_cavekeeper(), Some((4, 3)), vec![Keyword::Bushido(2)]),
        ("Mass of Ghouls", catalog::mass_of_ghouls(), Some((5, 3)), vec![]),
        ("Hand of Cruelty", catalog::hand_of_cruelty(), None,
            vec![Keyword::Protection(Color::White), Keyword::Bushido(1)]),
        ("Hand of Honor", catalog::hand_of_honor(), None, vec![Keyword::Protection(Color::Black)]),
        ("Gnarled Mass", catalog::gnarled_mass(), Some((3, 3)), vec![]),
        ("Humble Budoka", catalog::humble_budoka(), None, vec![Keyword::Shroud]),
        ("Mystic Restraints", catalog::mystic_restraints(), None, vec![Keyword::Flash]),
    ];
    for (name, d, pt, kws) in rows {
        if let Some(pt) = pt {
            assert_eq!((d.power, d.toughness), pt, "{name} P/T");
        }
        for k in kws {
            assert!(d.keywords.contains(&k), "{name} missing a printed keyword");
        }
    }
    // Non-keyword catalog checks folded in from individual tests.
    assert_eq!(catalog::numai_outcast().activated_abilities.len(), 1, "Numai regenerate ability");
    assert!(!catalog::vine_kami().triggered_abilities.is_empty(), "Vine Kami Soulshift");
    assert!(!catalog::kami_of_empty_graves().triggered_abilities.is_empty(), "Kami of Empty Graves Soulshift");
    assert!(matches!(catalog::wear_away().keywords[0], Keyword::Splice(_, _)), "Wear Away Splice");
    assert!(catalog::dampen_thought().keywords.iter().any(|k| matches!(k, Keyword::Splice(..))), "Dampen Thought Splice");
    assert!(catalog::throat_slitter().keywords.iter().any(|k| matches!(k, Keyword::Ninjutsu(_))), "Throat Slitter Ninjutsu");
}

// ── Soulshift table ──────────────────────────────────────────────────────────

/// Soulshift returns a Spirit with small-enough MV from the graveyard when the
/// carrier dies (Gibbering Kami 3, Crawling Filth 5, Nightsoil Kami 5,
/// Promised Kannushi 7).
#[test]
fn soulshift_returns_spirit_on_death() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    for (name, carrier, spirit) in [
        ("Gibbering Kami", catalog::gibbering_kami(), catalog::kami_of_ancient_law()),
        ("Crawling Filth", catalog::crawling_filth(), catalog::gnarled_mass()),
        ("Nightsoil Kami", catalog::nightsoil_kami(), catalog::kami_of_ancient_law()),
        ("Promised Kannushi", catalog::promised_kannushi(), catalog::gibbering_kami()),
    ] {
        let mut g = two_player_game();
        let c = g.add_card_to_battlefield(0, carrier);
        let s = g.add_card_to_graveyard(0, spirit);
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
        let events = g.remove_to_graveyard_with_triggers(c);
        g.dispatch_triggers_for_events(&events);
        drain_stack(&mut g);
        assert!(g.players[0].hand.iter().any(|card| card.id == s),
            "{name}: soulshift returned the Spirit");
    }
}

// ── Targeted removal spells table ────────────────────────────────────────────

/// One-shot removal spells: cast at a permanent and it leaves the battlefield.
#[test]
fn targeted_removal_spells() {
    use crabomination::mana::Color::*;
    let rows = [
        ("Befoul", catalog::befoul(), catalog::forest(), false, vec![(Black, 2)], 2),
        ("Rend Flesh", catalog::rend_flesh(), catalog::grizzly_bears(), false, vec![(Black, 1)], 2),
        ("Rend Spirit", catalog::rend_spirit(), catalog::lantern_kami(), false, vec![(Black, 1)], 2),
        ("Ghostly Visit", catalog::ghostly_visit(), catalog::grizzly_bears(), false, vec![(Black, 1)], 2),
        ("Pull Under", catalog::pull_under(), catalog::grizzly_bears(), false, vec![(Black, 1)], 5),
        ("Kiku's Shadow", catalog::kikus_shadow(), catalog::grizzly_bears(), false, vec![(Black, 2)], 0),
        ("Gut Shot", catalog::gut_shot(), catalog::frostling(), false, vec![(Red, 1)], 0),
        ("Quiet Purity", catalog::quiet_purity(), catalog::concordant_crossroads(), false, vec![(White, 1)], 0),
        ("Wear Away", catalog::wear_away(), catalog::sol_ring(), false, vec![(Green, 2)], 0),
        ("Crushing Pain", catalog::crushing_pain(), catalog::grizzly_bears(), true, vec![(Red, 1)], 1),
    ];
    for (name, spell, victim, pre_damaged, mana, colorless) in rows {
        let mut g = two_player_game();
        let v = g.add_card_to_battlefield(1, victim);
        if pre_damaged {
            g.battlefield_find_mut(v).unwrap().dealt_damage_this_turn = true;
        }
        let s = g.add_card_to_hand(0, spell);
        for (c, n) in mana { g.players[0].mana_pool.add(c, n); }
        g.players[0].mana_pool.add_colorless(colorless);
        g.perform_action(GameAction::CastSpell {
            card_id: s, target: Some(Target::Permanent(v)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect(name);
        drain_stack(&mut g);
        g.check_state_based_actions();
        assert!(g.battlefield_find(v).is_none(), "{name}: target removed");
    }
}

// ── Targeted activated abilities table ───────────────────────────────────────

/// Activated abilities (index 0) with a straightforward outcome, one row per
/// card: source on seat 0, optional victim, mana, then a per-card check.
#[test]
fn simple_activated_abilities() {
    use crabomination::mana::Color::*;
    let rows: Vec<(&str, _, Option<(_, _)>, Vec<(_, _)>, _, fn(&GameState, _, _))> = vec![
        ("Kabuto Moth", catalog::kabuto_moth(), Some((0, catalog::grizzly_bears())), vec![], 0,
            |g, s, t| {
                let cp = g.computed_permanent(t).unwrap();
                assert_eq!((cp.power, cp.toughness), (3, 4), "+1/+2 applied");
                assert!(g.battlefield_find(s).unwrap().tapped, "Moth tapped for the ability");
            }),
        ("Kitsune Diviner", catalog::kitsune_diviner(), Some((1, catalog::lantern_kami())), vec![], 0,
            |g, _, t| assert!(g.battlefield_find(t).unwrap().tapped, "Spirit tapped")),
        ("Mothrider Patrol", catalog::mothrider_patrol(), Some((1, catalog::grizzly_bears())),
            vec![(White, 1)], 3,
            |g, _, t| assert!(g.battlefield_find(t).unwrap().tapped, "target creature tapped")),
        ("Frostwielder", catalog::frostwielder(), Some((1, catalog::frostling())), vec![], 0,
            |g, _, t| assert!(g.battlefield_find(t).is_none(), "1/1 pinged dead")),
        ("Kiku, Night's Flower", catalog::kiku_nights_flower(), Some((1, catalog::grizzly_bears())),
            vec![(Black, 2)], 2,
            |g, _, t| assert!(g.battlefield_find(t).is_none(), "2/2 dealt itself 2 and died")),
        ("Akki Drillmaster", catalog::akki_drillmaster(), Some((0, catalog::grizzly_bears())), vec![], 0,
            |g, _, t| assert!(g.computed_permanent(t).unwrap().keywords.contains(&Keyword::Haste))),
        ("Matsu-Tribe Sniper", catalog::matsu_tribe_sniper(), Some((1, catalog::mahamoti_djinn())),
            vec![], 0,
            |g, _, t| {
                let d = g.battlefield_find(t).unwrap();
                assert_eq!(d.damage, 1, "1 damage pinged");
                assert!(d.tapped && d.skip_next_untap, "flyer tapped and locked");
            }),
        ("Nine-Ringed Bo", catalog::nine_ringed_bo(), Some((1, catalog::lantern_kami())), vec![], 0,
            |g, _, t| {
                assert!(g.battlefield_find(t).is_none(), "1/1 dies to the ping");
                assert!(g.exile.iter().any(|c| c.id == t), "exiled, not graveyard");
            }),
        ("Cursed Ronin", catalog::cursed_ronin(), None, vec![(Black, 1)], 0,
            |g, s, _| assert_eq!(g.computed_permanent(s).unwrap().power, 2, "+1/+1 firebreath")),
        ("Kuro's Taken", catalog::kuros_taken(), None, vec![(Black, 1)], 1,
            |g, s, _| assert!(g.battlefield_find(s).unwrap().regeneration_shields >= 1, "regen shield")),
        ("Heartless Hidetsugu", catalog::heartless_hidetsugu(), None, vec![], 0,
            |g, _, _| {
                assert_eq!(g.players[0].life, 10, "20 → took 10 damage");
                assert_eq!(g.players[1].life, 10, "20 → took 10 damage");
            }),
        ("Orochi Sustainer", catalog::orochi_sustainer(), None, vec![], 0,
            |g, _, _| assert_eq!(g.players[0].mana_pool.amount(crabomination::mana::Color::Green), 1)),
    ];
    for (name, src_def, victim, mana, colorless, check) in rows {
        let mut g = two_player_game();
        let src = g.add_card_to_battlefield(0, src_def);
        g.clear_sickness(src);
        let tgt = victim.map(|(seat, d)| g.add_card_to_battlefield(seat, d));
        for (c, n) in mana { g.players[0].mana_pool.add(c, n); }
        g.players[0].mana_pool.add_colorless(colorless);
        g.perform_action(GameAction::ActivateAbility {
            card_id: src, ability_index: 0, target: tgt.map(Target::Permanent),
            additional_targets: Vec::new(), x_value: None,
        }).expect(name);
        drain_stack(&mut g);
        g.check_state_based_actions();
        check(&g, src, tgt.unwrap_or(src));
    }
}

// ── Sacrifice-cost activated abilities ───────────────────────────────────────

/// Sacrifice-cost activated abilities, one mini-scenario per card.
#[test]
fn sacrifice_activated_abilities() {
    use crabomination::mana::Color::*;
    // Kami of Ancient Law: sac to destroy an enchantment.
    {
        let mut g = two_player_game();
        let kami = g.add_card_to_battlefield(0, catalog::kami_of_ancient_law());
        let ench = g.add_card_to_battlefield(1, catalog::concordant_crossroads());
        g.clear_sickness(kami);
        act!(g, kami, 0, Some(Target::Permanent(ench)));
        assert!(g.battlefield_find(kami).is_none() && g.battlefield_find(ench).is_none(),
            "Kami sacrificed as a cost, enchantment destroyed");
    }
    // Kami of Twisted Reflection: sac to bounce your own creature.
    {
        let mut g = two_player_game();
        let kami = g.add_card_to_battlefield(0, catalog::kami_of_twisted_reflection());
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.clear_sickness(kami);
        act!(g, kami, 0, Some(Target::Permanent(bear)));
        assert!(g.battlefield_find(kami).is_none());
        assert!(g.players[0].hand.iter().any(|c| c.id == bear), "your creature returned to hand");
    }
    // Pain Kami: {X}{R}, Sac → X damage to a creature.
    {
        let mut g = two_player_game();
        let kami = g.add_card_to_battlefield(0, catalog::pain_kami());
        let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        g.clear_sickness(kami);
        g.players[0].mana_pool.add(Red, 1);
        g.players[0].mana_pool.add_colorless(2);
        act!(g, kami, 0, Some(Target::Permanent(bear)), x = 2);
        assert!(g.battlefield_find(kami).is_none() && g.battlefield_find(bear).is_none(),
            "Pain Kami sacrificed; 2 damage killed the 2/2");
    }
    // Frostling: sac to ping for 1.
    {
        let mut g = two_player_game();
        let frost = g.add_card_to_battlefield(0, catalog::frostling());
        let target = g.add_card_to_battlefield(1, catalog::frostling());
        g.clear_sickness(frost);
        act!(g, frost, 0, Some(Target::Permanent(target)));
        assert!(g.battlefield_find(frost).is_none() && g.battlefield_find(target).is_none());
    }
    // Hearth Kami: {X}, Sac → destroy an artifact with MV X.
    {
        let mut g = two_player_game();
        let kami = g.add_card_to_battlefield(0, catalog::hearth_kami());
        let ring = g.add_card_to_battlefield(1, catalog::sol_ring()); // MV 1
        g.clear_sickness(kami);
        g.players[0].mana_pool.add_colorless(1);
        act!(g, kami, 0, Some(Target::Permanent(ring)), x = 1);
        assert!(g.battlefield_find(ring).is_none(), "Sol Ring (MV 1) destroyed for X=1");
    }
    // Scuttling Death: sac to shrink a creature -1/-1.
    {
        let mut g = two_player_game();
        let death = g.add_card_to_battlefield(0, catalog::scuttling_death());
        let victim = g.add_card_to_battlefield(1, catalog::frostling());
        g.clear_sickness(death);
        act!(g, death, 0, Some(Target::Permanent(victim)));
        assert!(g.battlefield_find(death).is_none() && g.battlefield_find(victim).is_none(),
            "1/1 shrank to 0/0 and died");
    }
    // Bile Urchin: sac to drain a player 1 life.
    {
        let mut g = two_player_game();
        let urchin = g.add_card_to_battlefield(0, catalog::bile_urchin());
        g.clear_sickness(urchin);
        let before = g.players[1].life;
        act!(g, urchin, 0, Some(Target::Player(1)));
        assert!(g.battlefield_find(urchin).is_none());
        assert_eq!(g.players[1].life, before - 1, "target lost 1 life");
    }
    // Burr Grafter: sac to give +2/+2.
    {
        let mut g = two_player_game();
        let grafter = g.add_card_to_battlefield(0, catalog::burr_grafter());
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.clear_sickness(grafter);
        act!(g, grafter, 0, Some(Target::Permanent(bear)));
        assert!(g.battlefield_find(grafter).is_none());
        let cp = g.computed_permanent(bear).unwrap();
        assert_eq!((cp.power, cp.toughness), (4, 4), "+2/+2 applied");
    }
    // Child of Thorns: sac to give +1/+1.
    {
        let mut g = two_player_game();
        let child = g.add_card_to_battlefield(0, catalog::child_of_thorns());
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        act!(g, child, 0, Some(Target::Permanent(bear)));
        assert!(g.battlefield_find(child).is_none());
        assert_eq!(g.battlefield_find(bear).unwrap().power(), 3, "bear pumped +1/+1");
    }
    // Moonlit Strider: sac to grant protection from a chosen color.
    {
        let mut g = two_player_game();
        let strider = g.add_card_to_battlefield(0, catalog::moonlit_strider());
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        act!(g, strider, 0, Some(Target::Permanent(bear)));
        assert!(g.battlefield_find(strider).is_none());
        assert!(g.computed_permanent(bear).unwrap().keywords.iter()
            .any(|k| matches!(k, Keyword::Protection(_))), "bear gained protection from a color");
    }
    // Pus Kami: sac to destroy a nonblack creature.
    {
        let mut g = two_player_game();
        let kami = g.add_card_to_battlefield(0, catalog::pus_kami());
        let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        g.players[0].mana_pool.add(Black, 1);
        act!(g, kami, 0, Some(Target::Permanent(bear)));
        assert!(g.battlefield_find(kami).is_none() && g.battlefield_find(bear).is_none());
    }
    // Teardrop Kami: sac to tap a creature (mode 0).
    {
        let mut g = two_player_game();
        let kami = g.add_card_to_battlefield(0, catalog::teardrop_kami());
        let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        g.clear_sickness(kami);
        g.decider = Box::new(crabomination::decision::ScriptedDecider::new(vec![
            crabomination::decision::DecisionAnswer::Mode(0),
        ]));
        act!(g, kami, 0, Some(Target::Permanent(bear)));
        assert!(g.battlefield_find(kami).is_none());
        assert!(g.battlefield_find(bear).unwrap().tapped, "target creature tapped");
    }
    // Hana Kami: sac to return an Arcane card from the graveyard.
    {
        let mut g = two_player_game();
        let hana = g.add_card_to_battlefield(0, catalog::hana_kami());
        let ray = g.add_card_to_graveyard(0, catalog::glacial_ray());
        g.clear_sickness(hana);
        g.players[0].mana_pool.add(Green, 1);
        g.players[0].mana_pool.add_colorless(1);
        act!(g, hana, 0, Some(Target::Permanent(ray)));
        assert!(g.battlefield_find(hana).is_none());
        assert!(g.players[0].hand.iter().any(|c| c.id == ray), "Arcane card returned to hand");
    }
    // He Who Hungers: sac a Spirit to strip a card from an opponent's hand.
    {
        let mut g = two_player_game();
        let hwh = g.add_card_to_battlefield(0, catalog::he_who_hungers());
        g.add_card_to_battlefield(0, catalog::gibbering_kami()); // Spirit fodder
        g.add_card_to_hand(1, catalog::lightning_bolt());
        g.clear_sickness(hwh);
        g.players[0].mana_pool.add_colorless(1);
        act!(g, hwh, 0, Some(Target::Player(1)));
        assert_eq!(g.players[1].hand.len(), 0, "opponent discarded the chosen card");
    }
    // Nezumi Shadow-Watcher: sac to destroy a Ninja.
    {
        let mut g = two_player_game();
        let watcher = g.add_card_to_battlefield(0, catalog::nezumi_shadow_watcher());
        let ninja = g.add_card_to_battlefield(1, {
            let mut d = catalog::grizzly_bears();
            d.name = "Ninja";
            d.subtypes.creature_types = vec![crabomination::card::CreatureType::Ninja];
            d
        });
        g.clear_sickness(watcher);
        act!(g, watcher, 0, Some(Target::Permanent(ninja)));
        assert!(g.battlefield_find(ninja).is_none(), "Ninja destroyed");
        assert!(g.players[0].graveyard.iter().any(|c| c.id == watcher), "Watcher sacrificed");
    }
    // Akki Avalanchers: sac a land to pump itself +2/+0.
    {
        let mut g = two_player_game();
        let akki = g.add_card_to_battlefield(0, catalog::akki_avalanchers());
        let land = g.add_card_to_battlefield(0, catalog::mountain());
        g.clear_sickness(akki);
        act!(g, akki, 0, None);
        assert!(g.battlefield_find(land).is_none(), "land sacrificed");
        assert_eq!(g.computed_permanent(akki).unwrap().power, 3, "+2/+0 → 3 power");
    }
    // Foratog: sac a Forest to get +2/+2.
    {
        let mut g = two_player_game();
        let atog = g.add_card_to_battlefield(0, catalog::foratog());
        let forest = g.add_card_to_battlefield(0, catalog::forest());
        g.clear_sickness(atog);
        g.players[0].mana_pool.add(Green, 1);
        act!(g, atog, 0, None);
        assert!(g.battlefield_find(forest).is_none(), "Forest sacrificed");
        assert_eq!(g.battlefield_find(atog).unwrap().power(), 3, "Foratog is 3/4");
    }
    // Nezumi Bone-Reader: sac a creature to make a player discard.
    {
        let mut g = two_player_game();
        let reader = g.add_card_to_battlefield(0, catalog::nezumi_bone_reader());
        g.add_card_to_battlefield(0, catalog::frostling()); // fodder
        g.add_card_to_hand(1, catalog::grizzly_bears());
        g.clear_sickness(reader);
        g.players[0].mana_pool.add(Black, 1);
        act!(g, reader, 0, Some(Target::Player(1)));
        assert_eq!(g.players[1].hand.len(), 0, "target player discarded their card");
    }
}

// ── CHK gap batch 1 (`decks::recent325`) ──

mod gaps1 {
    use crabomination::card::{CounterType, Keyword};
    use crabomination::catalog;
    use crabomination::game::types::{GameAction, Target, TurnStep};
    use crabomination::game::*;
    use crabomination::mana::Color;

    fn main_phase() -> GameState {
        let mut g = two_player_game();
        g.active_player_idx = 0;
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g
    }

    /// The slow-dual cycle taps for colourless freely, or for colour at the
    /// cost of its next untap.
    #[test]
    fn slow_duals_trade_an_untap_for_colour() {
        for def in [
            catalog::cloudcrest_lake(),
            catalog::lantern_lit_graveyard(),
            catalog::pinecrest_ridge(),
            catalog::tranquil_garden(),
            catalog::waterveil_cavern(),
        ] {
            let name = def.name;
            let mut g = main_phase();
            let land = g.add_card_to_battlefield(0, def);
            g.perform_action(GameAction::ActivateAbility {
                card_id: land, ability_index: 1, target: None, additional_targets: vec![],
                x_value: None,
            })
            .expect("colour tap");
            drain_stack(&mut g);
            assert_eq!(g.players[0].mana_pool.total(), 1, "{name}");
            g.do_untap();
            assert!(g.battlefield_find(land).unwrap().tapped, "{name} skipped its untap");
        }
    }

    /// Forbidden Orchard pays out any colour and gifts them a Spirit.
    #[test]
    fn forbidden_orchard_gifts_a_spirit() {
        let mut g = main_phase();
        let land = g.add_card_to_battlefield(0, catalog::forbidden_orchard());
        g.perform_action(GameAction::ActivateAbility {
            card_id: land, ability_index: 0, target: Some(Target::Player(1)),
            additional_targets: vec![], x_value: None,
        })
        .expect("tap");
        drain_stack(&mut g);
        assert_eq!(g.players[0].mana_pool.total(), 1);
        assert_eq!(
            g.battlefield.iter().filter(|c| c.definition.name == "Spirit" && c.controller == 1).count(),
            1
        );
    }

    /// Untaidake's mana only pays for legendary spells.
    #[test]
    fn untaidake_mana_is_legendary_only() {
        let mut g = main_phase();
        let land = g.add_card_to_battlefield(0, catalog::untaidake_the_cloud_keeper());
        g.battlefield_find_mut(land).unwrap().tapped = false;
        g.perform_action(GameAction::ActivateAbility {
            card_id: land, ability_index: 0, target: None, additional_targets: vec![],
            x_value: None,
        })
        .expect("tap for two");
        assert_eq!(g.players[0].life, 18);
        let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
        assert!(
            g.perform_action(GameAction::CastSpell {
                card_id: bear, target: None, additional_targets: vec![], mode: None, x_value: None,
            })
            .is_err(),
            "a nonlegendary spell can't be paid for"
        );
    }

    /// A Myojin cast from hand is indestructible until it spends its counter.
    #[test]
    fn myojin_is_indestructible_until_it_fires() {
        let mut g = main_phase();
        let myojin = g.add_card_to_hand(0, catalog::myojin_of_cleansing_fire());
        let bystander = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        g.players[0].mana_pool.add(Color::White, 3);
        g.players[0].mana_pool.add_colorless(5);
        g.perform_action(GameAction::CastSpell {
            card_id: myojin, target: None, additional_targets: vec![], mode: None, x_value: None,
        })
        .expect("cast");
        drain_stack(&mut g);
        assert_eq!(g.battlefield_find(myojin).unwrap().counter_count(CounterType::Divinity), 1);
        assert!(
            g.computed_permanent(myojin).unwrap().keywords.contains(&Keyword::Indestructible)
        );
        g.perform_action(GameAction::ActivateAbility {
            card_id: myojin, ability_index: 0, target: None, additional_targets: vec![],
            x_value: None,
        })
        .expect("wrath");
        drain_stack(&mut g);
        assert!(g.battlefield_find(bystander).is_none(), "everything else died");
        assert!(g.battlefield_find(myojin).is_some(), "the Myojin survives its own wrath");
    }

    /// A Myojin that didn't come from hand gets no divinity counter.
    #[test]
    fn myojin_reanimated_has_no_divinity_counter() {
        let mut g = main_phase();
        let myojin = g.add_card_to_battlefield(0, catalog::myojin_of_seeing_winds());
        assert_eq!(g.battlefield_find(myojin).unwrap().counter_count(CounterType::Divinity), 0);
        assert!(
            !g.computed_permanent(myojin).unwrap().keywords.contains(&Keyword::Indestructible)
        );
    }

    /// Azami taps your Wizards for cards.
    #[test]
    fn azami_taps_wizards_for_cards() {
        let mut g = main_phase();
        let azami = g.add_card_to_battlefield(0, catalog::azami_lady_of_scrolls());
        let wizard = g.add_card_to_battlefield(0, catalog::azami_lady_of_scrolls());
        g.clear_sickness(wizard);
        g.add_card_to_library(0, catalog::forest());
        let before = g.players[0].hand.len();
        g.perform_action(GameAction::ActivateAbility {
            card_id: azami, ability_index: 0, target: None, additional_targets: vec![],
            x_value: None,
        })
        .expect("draw");
        drain_stack(&mut g);
        assert_eq!(g.players[0].hand.len(), before + 1);
        assert!(g.battlefield_find(wizard).unwrap().tapped);
    }

    /// Night of Souls' Betrayal shrinks the whole board.
    #[test]
    fn night_of_souls_betrayal_shrinks_everyone() {
        let mut g = main_phase();
        g.add_card_to_battlefield(0, catalog::night_of_souls_betrayal());
        let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        for id in [mine, theirs] {
            let cp = g.computed_permanent(id).unwrap();
            assert_eq!((cp.power, cp.toughness), (1, 1));
        }
    }

    /// Mana Seism converts sacrificed lands into colourless mana.
    #[test]
    fn mana_seism_cashes_in_lands() {
        let mut g = main_phase();
        for _ in 0..3 {
            g.add_card_to_battlefield(0, catalog::forest());
        }
        let spell = g.add_card_to_hand(0, catalog::mana_seism());
        g.players[0].mana_pool.add(Color::Red, 1);
        g.players[0].mana_pool.add_colorless(1);
        g.perform_action(GameAction::CastSpell {
            card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: Some(3),
        })
        .expect("cast");
        drain_stack(&mut g);
        assert_eq!(g.players[0].mana_pool.total(), 3);
        assert_eq!(g.battlefield.iter().filter(|c| c.definition.is_land()).count(), 0);
    }

    /// Devouring Rage adds +3/+0 per Spirit fed to it.
    #[test]
    fn devouring_rage_scales_with_spirits() {
        let mut g = main_phase();
        let target = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        for _ in 0..2 {
            g.add_card_to_battlefield(0, catalog::ore_gorger());
        }
        let spell = g.add_card_to_hand(0, catalog::devouring_rage());
        g.players[0].mana_pool.add(Color::Red, 1);
        g.players[0].mana_pool.add_colorless(4);
        g.perform_action(GameAction::CastSpell {
            card_id: spell, target: Some(Target::Permanent(target)), additional_targets: vec![],
            mode: None, x_value: Some(2),
        })
        .expect("cast");
        drain_stack(&mut g);
        assert_eq!(g.computed_permanent(target).unwrap().power, 2 + 3 + 6);
    }

    /// Thoughtbind only answers cheap spells.
    #[test]
    fn thoughtbind_caps_at_mana_value_four() {
        let mut g = main_phase();
        let big = g.add_card_to_hand(1, catalog::myojin_of_seeing_winds());
        g.players[1].mana_pool.add(Color::Blue, 3);
        g.players[1].mana_pool.add_colorless(7);
        g.active_player_idx = 1;
        g.priority.player_with_priority = 1;
        g.perform_action(GameAction::CastSpell {
            card_id: big, target: None, additional_targets: vec![], mode: None, x_value: None,
        })
        .expect("cast the fatty");
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        let bind = g.add_card_to_hand(0, catalog::thoughtbind());
        g.players[0].mana_pool.add(Color::Blue, 1);
        g.players[0].mana_pool.add_colorless(2);
        assert!(
            g.perform_action(GameAction::CastSpell {
                card_id: bind, target: Some(Target::Permanent(big)), additional_targets: vec![],
                mode: None, x_value: None,
            })
            .is_err(),
            "mana value 10 is out of range"
        );
    }

    /// Imi Statue caps each player's artifact untaps at one.
    #[test]
    fn imi_statue_caps_artifact_untaps() {
        let mut g = main_phase();
        g.add_card_to_battlefield(0, catalog::imi_statue());
        let a = g.add_card_to_battlefield(0, catalog::hair_strung_koto());
        let b = g.add_card_to_battlefield(0, catalog::honor_worn_shaku());
        for id in [a, b] {
            g.battlefield_find_mut(id).unwrap().tapped = true;
        }
        g.do_untap();
        let untapped = [a, b].iter().filter(|id| !g.battlefield_find(**id).unwrap().tapped).count();
        assert_eq!(untapped, 1, "only one artifact untapped");
    }

    /// Orochi Hatchery hatches a Snake per charge counter.
    #[test]
    fn orochi_hatchery_hatches_per_counter() {
        let mut g = main_phase();
        let hatch = g.add_card_to_hand(0, catalog::orochi_hatchery());
        g.players[0].mana_pool.add_colorless(4);
        g.perform_action(GameAction::CastSpell {
            card_id: hatch, target: None, additional_targets: vec![], mode: None, x_value: Some(2),
        })
        .expect("cast for X=2");
        drain_stack(&mut g);
        assert_eq!(g.battlefield_find(hatch).unwrap().counter_count(CounterType::Charge), 2);
        g.clear_sickness(hatch);
        g.players[0].mana_pool.add_colorless(5);
        g.perform_action(GameAction::ActivateAbility {
            card_id: hatch, ability_index: 0, target: None, additional_targets: vec![],
            x_value: None,
        })
        .expect("hatch");
        drain_stack(&mut g);
        assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Snake").count(), 2);
    }

    /// Tenza is bigger on a legend and tramples in red.
    #[test]
    fn tenza_scales_with_the_host() {
        let mut g = main_phase();
        let tenza = g.add_card_to_battlefield(0, catalog::tenza_godos_maul());
        let plain = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.players[0].mana_pool.add_colorless(1);
        g.perform_action(GameAction::Equip { equipment: tenza, target: plain }).expect("equip");
        assert_eq!(g.computed_permanent(plain).unwrap().power, 3, "+1/+1 on a plain green bear");
        let legend = g.add_card_to_battlefield(0, catalog::azami_lady_of_scrolls());
        g.players[0].mana_pool.add_colorless(1);
        g.perform_action(GameAction::Equip { equipment: tenza, target: legend }).expect("re-equip");
        assert_eq!(g.computed_permanent(legend).unwrap().power, 3, "0/2 + 1 + 2 legendary");
    }

    /// Sachi tops up Snakes and turns Shamans into mana sources.
    #[test]
    fn sachi_pumps_snakes_and_taps_shamans() {
        let mut g = main_phase();
        let sachi = g.add_card_to_battlefield(0, catalog::sachi_daughter_of_seshiro());
        g.clear_sickness(sachi);
        // Sachi is herself a Shaman, so she taps for {G}{G}; the anthem
        // excludes her.
        assert_eq!(g.computed_permanent(sachi).unwrap().toughness, 3);
        g.perform_action(GameAction::ActivateAbility {
            card_id: sachi, ability_index: 0, target: None, additional_targets: vec![],
            x_value: None,
        })
        .expect("tap for GG");
        assert_eq!(g.players[0].mana_pool.total(), 2);
    }

    /// Ragged Veins bleeds the host's controller for every point it takes.
    #[test]
    fn ragged_veins_mirrors_damage_to_life() {
        let mut g = main_phase();
        let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let aura = g.add_card_to_hand(0, catalog::ragged_veins());
        g.players[0].mana_pool.add(Color::Black, 1);
        g.players[0].mana_pool.add_colorless(1);
        g.perform_action(GameAction::CastSpell {
            card_id: aura, target: Some(Target::Permanent(bear)), additional_targets: vec![],
            mode: None, x_value: None,
        })
        .expect("cast");
        drain_stack(&mut g);
        let mut evs = Vec::new();
        g.deal_damage_to_from(
            crabomination::game::effects::EntityRef::Permanent(bear), 1, None, &mut evs,
        );
        g.dispatch_triggers_for_events(&evs);
        drain_stack(&mut g);
        assert_eq!(g.players[1].life, 19);
    }
}
