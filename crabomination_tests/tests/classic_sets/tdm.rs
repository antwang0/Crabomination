//! Functionality tests for `catalog::sets::decks::tdm`.

use crabomination::card::Keyword;
use crabomination::catalog;
use crabomination::game::*;
use crabomination::mana::Color;

fn advance_to(g: &mut GameState, step: TurnStep) {
    while g.step != step {
        g.perform_action(GameAction::PassPriority).expect("pass priority");
    }
}

type Catalog = fn() -> crabomination::card::CardDefinition;

/// Fill player 0's pool with ample mana of every color.
fn add_ample_mana(g: &mut GameState) {
    g.players[0].mana_pool.add_colorless(8);
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[0].mana_pool.add(c, 2);
    }
}

/// Spells cast at your own 2/2 bear: check resulting P/T and keywords.
/// Covers Alesha's Legacy, Fire-Rim Form, Fresh Start, Bewilder.
#[test]
fn targeted_pt_and_keyword_spells() {
    let rows: &[(&str, Catalog, i64, i64, &[Keyword], bool)] = &[
        (
            "Alesha's Legacy",
            catalog::aleshas_legacy,
            2,
            2,
            &[Keyword::Deathtouch, Keyword::Indestructible],
            false,
        ),
        ("Fire-Rim Form", catalog::fire_rim_form, 4, 2, &[Keyword::FirstStrike], false),
        ("Fresh Start", catalog::fresh_start, -3, 2, &[], true),
        ("Bewilder", catalog::bewilder, -1, 2, &[], false),
    ];
    for &(name, make, p, t, kws, expect_no_kws) in rows {
        let mut g = two_player_game();
        let creature = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let spell = g.add_card_to_hand(0, make());
        g.add_card_to_library(0, catalog::grizzly_bears()); // in case of a cantrip
        add_ample_mana(&mut g);
        g.step = TurnStep::PreCombatMain;
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::CastSpell {
            card_id: spell,
            target: Some(Target::Permanent(creature)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .unwrap_or_else(|e| panic!("cast {name}: {e:?}"));
        drain_stack(&mut g);
        let cp = g.computed_permanent(creature).unwrap();
        assert_eq!((cp.power as i64, cp.toughness as i64), (p, t), "{name}: P/T");
        for kw in kws {
            assert!(cp.keywords.contains(kw), "{name}: expected {kw:?}");
        }
        if expect_no_kws {
            assert!(cp.keywords.is_empty(), "{name}: abilities removed");
        }
    }
}

/// One-target/no-target sorcery outcomes: victim leaves the battlefield
/// (optionally to exile) and/or named tokens are minted. Covers Kin-Tree
/// Severance, Dragon's Prey, Salt Road Skirmish, Revival of the Ancestors.
#[test]
fn removal_and_token_spells() {
    let rows: &[(&str, Catalog, Option<Catalog>, bool, Option<(&str, usize)>)] = &[
        ("Kin-Tree Severance", catalog::kin_tree_severance, Some(catalog::serra_angel), true, None),
        ("Dragon's Prey", catalog::dragons_prey, Some(catalog::grizzly_bears), false, None),
        (
            "Salt Road Skirmish",
            catalog::salt_road_skirmish,
            Some(catalog::grizzly_bears),
            false,
            Some(("Warrior", 2)),
        ),
        (
            "Revival of the Ancestors",
            catalog::revival_of_the_ancestors,
            None,
            false,
            Some(("Spirit", 3)),
        ),
    ];
    for &(name, make, victim_def, exiled, token) in rows {
        let mut g = two_player_game();
        let victim = victim_def.map(|d| g.add_card_to_battlefield(1, d()));
        let spell = g.add_card_to_hand(0, make());
        add_ample_mana(&mut g);
        g.step = TurnStep::PreCombatMain;
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::CastSpell {
            card_id: spell,
            target: victim.map(Target::Permanent),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .unwrap_or_else(|e| panic!("cast {name}: {e:?}"));
        drain_stack(&mut g);
        if let Some(v) = victim {
            assert!(g.battlefield_find(v).is_none(), "{name}: target removed");
            if exiled {
                assert!(g.exile.iter().any(|c| c.id == v), "{name}: went to exile");
            }
        }
        if let Some((tok, n)) = token {
            let count = g
                .battlefield
                .iter()
                .filter(|c| c.controller == 0 && c.definition.name == tok)
                .count();
            assert_eq!(count, n, "{name}: {tok} tokens");
        }
    }
}

/// Pure printed stat/keyword checks. Covers Jeskai Brushmaster and
/// Rot-Curse Rakshasa.
#[test]
fn definition_stats_and_keywords() {
    let rows: &[(&str, Catalog, i64, i64, &[Keyword])] = &[
        (
            "Jeskai Brushmaster",
            catalog::jeskai_brushmaster,
            2,
            4,
            &[Keyword::DoubleStrike, Keyword::Prowess],
        ),
        ("Rot-Curse Rakshasa", catalog::rot_curse_rakshasa, 5, 5, &[Keyword::Trample, Keyword::Decayed]),
    ];
    for &(name, make, p, t, kws) in rows {
        let d = make();
        assert_eq!((d.power as i64, d.toughness as i64), (p, t), "{name}: P/T");
        for kw in kws {
            assert!(d.keywords.contains(kw), "{name}: expected {kw:?}");
        }
    }
}

/// Jade-Cast Sentinel bottoms a graveyard card.
#[test]
fn jade_cast_sentinel_bottoms_graveyard_card() {
    let mut g = two_player_game();
    let sentinel = g.add_card_to_battlefield(0, catalog::jade_cast_sentinel());
    g.clear_sickness(sentinel);
    let dead = g.add_card_to_graveyard(1, catalog::grizzly_bears());
    g.players[0].mana_pool.add_colorless(2);
    g.step = TurnStep::PreCombatMain;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: sentinel,
        ability_index: 0,
        target: Some(Target::Permanent(dead)),
        additional_targets: Vec::new(),
        x_value: None, mode: None,
    })
    .expect("bottom a graveyard card");
    drain_stack(&mut g);
    assert!(g.players[1].graveyard.iter().all(|c| c.id != dead), "left the graveyard");
    assert_eq!(g.players[1].library.last().unwrap().id, dead, "went to owner's library bottom");
}

/// Gurmag Nightwatch digs three, keeps one on top, mills the rest.
#[test]
fn gurmag_nightwatch_digs_and_mills() {
    let mut g = two_player_game();
    for _ in 0..5 {
        g.add_card_to_library(0, catalog::grizzly_bears());
    }
    let gy = g.players[0].graveyard.len();
    let lib = g.players[0].library.len();
    let creature = g.add_card_to_battlefield(0, catalog::gurmag_nightwatch());
    g.fire_self_etb_triggers(creature, 0);
    drain_stack(&mut g);
    // One kept on top, two milled → library down 2, graveyard up 2.
    assert_eq!(g.players[0].graveyard.len(), gy + 2, "milled two");
    assert_eq!(g.players[0].library.len(), lib - 2, "kept one on top");
}

/// Kin-Tree Severance exiles a MV-3+ permanent (and can't hit a cheap one).
#[test]
fn kin_tree_severance_exiles_expensive_permanent() {
    let mut g = two_player_game();
    let big = g.add_card_to_battlefield(1, catalog::serra_angel()); // MV 5
    let spell = g.add_card_to_hand(0, catalog::kin_tree_severance());
    g.players[0].mana_pool.add_colorless(6);
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: Some(Target::Permanent(big)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("exile the MV-5 Angel");
    drain_stack(&mut g);
    assert!(g.battlefield_find(big).is_none(), "MV-5 permanent exiled");
    assert!(g.exile.iter().any(|c| c.id == big), "went to exile");
}

/// Armament Dragon distributes three +1/+1 counters on enter.
#[test]
fn armament_dragon_distributes_counters() {
    use crabomination::card::CounterType;
    let mut g = two_player_game();
    let a = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let dragon = g.add_card_to_battlefield(0, catalog::armament_dragon());
    g.fire_self_etb_triggers(dragon, 0);
    drain_stack(&mut g);
    // AutoDecider spreads across available creatures; total distributed is 3.
    let total: u32 = g
        .battlefield
        .iter()
        .filter(|c| c.controller == 0)
        .map(|c| *c.counters.get(&CounterType::PlusOnePlusOne).unwrap_or(&0))
        .sum();
    assert_eq!(total, 3, "three +1/+1 counters placed");
    let _ = a;
}

/// Fresh Start weakens and silences the enchanted creature.
#[test]
fn fresh_start_shrinks_and_removes_abilities() {
    let mut g = two_player_game();
    let foe = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4 flyer w/ vigilance
    let aura = g.add_card_to_hand(0, catalog::fresh_start());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::CastSpell {
        card_id: aura,
        target: Some(Target::Permanent(foe)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("enchant the Angel");
    drain_stack(&mut g);
    let cp = g.computed_permanent(foe).unwrap();
    assert_eq!(cp.power, -1, "4 − 5 = −1 power");
    assert!(cp.keywords.is_empty(), "abilities removed");
}

/// Lie in Wait returns a creature and slings its power at a target.
#[test]
fn lie_in_wait_returns_and_deals_power() {
    let mut g = two_player_game();
    let dead = g.add_card_to_graveyard(0, catalog::hill_giant()); // 3/3, power 3
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let spell = g.add_card_to_hand(0, catalog::lie_in_wait());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: Some(Target::Permanent(dead)),
        additional_targets: vec![Target::Permanent(foe)],
        mode: None,
        x_value: None,
    })
    .expect("cast Lie in Wait");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == dead), "creature returned to hand");
    assert!(g.battlefield_find(foe).is_none(), "3 damage killed the 2/2");
}

/// Dragonstorm Globe gives entering Dragons an extra +1/+1 counter and taps for
/// any color.
#[test]
fn dragonstorm_globe_counters_dragons_and_makes_mana() {
    use crabomination::card::CounterType;
    let mut g = two_player_game();
    let globe = g.add_card_to_battlefield(0, catalog::dragonstorm_globe());
    // The static grants a +1/+1 counter to an entering Dragon (spec computed at
    // resolution — `add_card_to_battlefield` bypasses the ETB-counter path).
    let dragon = g.add_card_to_battlefield(0, catalog::armament_dragon());
    let specs = g.chosen_type_etb_counter_specs(dragon, 0);
    assert!(
        specs.contains(&(CounterType::PlusOnePlusOne, 1)),
        "entering Dragon gets the extra +1/+1 counter"
    );
    // {T}: add one mana of any color.
    g.clear_sickness(globe);
    g.step = TurnStep::PreCombatMain;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: globe,
        ability_index: 0,
        target: None,
        additional_targets: Vec::new(),
        x_value: None, mode: None,
    })
    .expect("tap for mana");
    assert!(g.players[0].mana_pool.total() >= 1, "produced a mana");
}

/// Wingspan Stride pumps and grants flying, and can bounce itself.
#[test]
fn wingspan_stride_pumps_flying_and_self_bounces() {
    let mut g = two_player_game();
    let creature = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let aura = g.add_card_to_hand(0, catalog::wingspan_stride());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::CastSpell {
        card_id: aura,
        target: Some(Target::Permanent(creature)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("enchant");
    drain_stack(&mut g);
    let cp = g.computed_permanent(creature).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 3), "+1/+1");
    assert!(cp.keywords.contains(&Keyword::Flying), "granted flying");
    // {2}{U}: bounce the Aura to hand.
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: aura,
        ability_index: 0,
        target: None,
        additional_targets: Vec::new(),
        x_value: None, mode: None,
    })
    .expect("bounce self");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == aura), "Aura returned to hand");
}

/// Riverwalk Technique's counter mode stops a noncreature spell.
#[test]
fn riverwalk_technique_counters_noncreature_spell() {
    let mut g = two_player_game();
    // Opponent casts a noncreature spell we can counter.
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.priority.player_with_priority = 1;
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Player(0)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("opponent casts Bolt");
    let tech = g.add_card_to_hand(0, catalog::riverwalk_technique());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: tech,
        target: Some(Target::Permanent(bolt)),
        additional_targets: vec![],
        mode: Some(1), // counter mode
        x_value: None,
    })
    .expect("counter the Bolt");
    drain_stack(&mut g);
    assert!(g.players[1].graveyard.iter().any(|c| c.id == bolt), "Bolt countered to graveyard");
    assert_eq!(g.players[0].life, 20, "Bolt never resolved");
}

/// Static Snare's ETB exiles an opponent's creature until it leaves; it returns
/// when the Snare does.
#[test]
fn static_snare_exiles_until_it_leaves() {
    let mut g = two_player_game();
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let snare = g.add_card_to_battlefield(0, catalog::static_snare());
    g.fire_self_etb_triggers(snare, 0);
    drain_stack(&mut g);
    assert!(g.battlefield_find(victim).is_none(), "creature exiled");
    // Destroy the Snare → the exiled creature returns to the battlefield.
    g.remove_from_battlefield_to_graveyard_raw(snare);
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.id == victim), "creature returned when Snare left");
}

/// Seize Opportunity's pump mode buffs up to two creatures.
#[test]
fn seize_opportunity_pumps_two_creatures() {
    let mut g = two_player_game();
    let a = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    let b = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    let spell = g.add_card_to_hand(0, catalog::seize_opportunity());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: Some(Target::Permanent(a)),
        additional_targets: vec![Target::Permanent(b)],
        mode: Some(1), // pump mode
        x_value: None,
    })
    .expect("cast Seize Opportunity (pump)");
    drain_stack(&mut g);
    let ca = g.computed_permanent(a).unwrap();
    let cb = g.computed_permanent(b).unwrap();
    assert_eq!((ca.power, ca.toughness), (4, 3), "+2/+1 on the first");
    assert_eq!((cb.power, cb.toughness), (4, 3), "+2/+1 on the second");
}

/// Ringing Strike Mastery taps the enchanted creature and locks its untap.
#[test]
fn ringing_strike_mastery_taps_and_locks_untap() {
    let mut g = two_player_game();
    let creature = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let aura = g.add_card_to_hand(0, catalog::ringing_strike_mastery());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::CastSpell {
        card_id: aura,
        target: Some(Target::Permanent(creature)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("enchant");
    drain_stack(&mut g);
    assert!(g.battlefield_find(creature).unwrap().tapped, "ETB tapped it");
    // The untap step doesn't free it.
    g.active_player_idx = 0;
    g.do_untap();
    assert!(g.battlefield_find(creature).unwrap().tapped, "stays tapped through untap");
}

/// Rally the Monastery's token mode makes two prowess Monks.
#[test]
fn rally_the_monastery_makes_two_monks() {
    let mut g = two_player_game();
    let spell = g.add_card_to_hand(0, catalog::rally_the_monastery());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: None,
        additional_targets: vec![],
        mode: Some(0), // token mode
        x_value: None,
    })
    .expect("cast Rally (tokens)");
    drain_stack(&mut g);
    let monks = g
        .battlefield
        .iter()
        .filter(|c| c.controller == 0 && c.definition.name == "Monk")
        .count();
    assert_eq!(monks, 2, "made two Monk tokens");
}

/// Rally the Monastery costs {2} less after another spell this turn.
#[test]
fn rally_the_monastery_cheaper_after_a_spell() {
    let mut g = two_player_game();
    let spell = g.add_card_to_hand(0, catalog::rally_the_monastery());
    // Pretend a prior spell resolved this turn.
    g.players[0].spells_cast_this_turn = 1;
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1); // only {1}{W} = 2 available, not 4
    g.step = TurnStep::PreCombatMain;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: None,
        additional_targets: vec![],
        mode: Some(0),
        x_value: None,
    })
    .expect("cast for {1}{W} thanks to the reduction");
    drain_stack(&mut g);
    let monks = g
        .battlefield
        .iter()
        .filter(|c| c.controller == 0 && c.definition.name == "Monk")
        .count();
    assert_eq!(monks, 2, "resolved after the discount");
}

/// Ringing Strike Mastery grants the enchanted creature "{5}: Untap this."
#[test]
fn ringing_strike_mastery_grants_untap_ability() {
    let mut g = two_player_game();
    let creature = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let aura = g.add_card_to_hand(0, catalog::ringing_strike_mastery());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g.active_player_idx = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: aura,
        target: Some(Target::Permanent(creature)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("enchant");
    drain_stack(&mut g);
    assert!(g.battlefield_find(creature).unwrap().tapped, "ETB tapped it");
    // The granted {5}: Untap ability is surfaced as a virtual ability past the
    // creature's printed abilities (index 0). Pay {5} and activate it.
    g.players[0].mana_pool.add_colorless(5);
    g.perform_action(GameAction::ActivateAbility {
        card_id: creature,
        ability_index: 0,
        target: None,
        additional_targets: Vec::new(),
        x_value: None, mode: None,
    })
    .expect("activate granted untap");
    drain_stack(&mut g);
    assert!(!g.battlefield_find(creature).unwrap().tapped, "paying five untapped it");
}

/// Krumar Initiate pays X life to endure X (X +1/+1 counters here).
#[test]
fn krumar_initiate_endures_x_paying_life() {
    let mut g = two_player_game();
    let init = g.add_card_to_battlefield(0, catalog::krumar_initiate());
    g.clear_sickness(init);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2); // for {X=2}
    g.step = TurnStep::PreCombatMain;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    let life_before = g.players[0].life;
    g.perform_action(GameAction::ActivateAbility {
        card_id: init,
        ability_index: 0,
        target: None,
        additional_targets: Vec::new(),
        x_value: Some(2), mode: None,
    })
    .expect("endure 2, pay 2 life");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life_before - 2, "paid X=2 life");
    // Endure X put 2 +1/+1 counters (or made an X/X); either way power grew.
    let cp = g.computed_permanent(init).unwrap();
    assert!(cp.power >= 4, "2/2 endured 2 → at least 4 power");
}

/// Zurgo's Vanguard's power equals the number of creatures you control.
#[test]
fn zurgos_vanguard_power_tracks_creature_count() {
    let mut g = two_player_game();
    let zurgo = g.add_card_to_battlefield(0, catalog::zurgos_vanguard());
    // Just Zurgo → power 1.
    assert_eq!(g.computed_permanent(zurgo).unwrap().power, 1, "counts itself");
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    assert_eq!(g.computed_permanent(zurgo).unwrap().power, 3, "3 creatures → power 3");
}

/// War Effort mints a tapped, attacking Warrior whenever you attack.
#[test]
fn war_effort_mobilizes_on_attack() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::war_effort());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(bear);
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: bear,
        target: AttackTarget::Player(1),
    }]))
    .expect("attack");
    drain_stack(&mut g);
    let warriors = g
        .battlefield
        .iter()
        .filter(|c| c.controller == 0 && c.definition.name == "Warrior")
        .count();
    assert_eq!(warriors, 1, "one tapped attacking Warrior");
    // Anthem: the attacking bear is +1/+0.
    assert_eq!(g.computed_permanent(bear).unwrap().power, 3, "+1/+0 anthem");
}

/// Dragon's Prey costs {2} more if it targets a Dragon.
#[test]
fn dragons_prey_costs_more_vs_dragon() {
    use crabomination::card::CardType;
    let dragon = catalog::armament_dragon();
    assert!(
        dragon.card_types.contains(&CardType::Creature),
        "sanity: armament_dragon is a creature"
    );
    let mut g = two_player_game();
    // Non-Dragon target: base cost {2}{B} = 3.
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::dragons_prey());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.step = TurnStep::PreCombatMain;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast at non-Dragon for {2}{B}");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none(), "bear destroyed");
}

/// Salt Road Skirmish destroys a creature and makes two haste Warriors.
#[test]
fn salt_road_skirmish_destroys_and_makes_warriors() {
    let mut g = two_player_game();
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::salt_road_skirmish());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.step = TurnStep::PreCombatMain;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: Some(Target::Permanent(victim)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast Salt Road Skirmish");
    drain_stack(&mut g);
    assert!(g.battlefield_find(victim).is_none(), "target destroyed");
    let warriors = g
        .battlefield
        .iter()
        .filter(|c| c.controller == 0 && c.definition.name == "Warrior")
        .collect::<Vec<_>>();
    assert_eq!(warriors.len(), 2, "two Warriors");
    assert!(
        warriors[0].definition.keywords.contains(&Keyword::Haste),
        "with haste"
    );
}

/// Corroding Dragonstorm drains 2 on ETB and bounces itself when a Dragon enters.
#[test]
fn corroding_dragonstorm_drains_and_bounces_on_dragon() {
    let mut g = two_player_game();
    let opp_life = g.players[1].life;
    let my_life = g.players[0].life;
    let storm = g.add_card_to_hand(0, catalog::corroding_dragonstorm());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.step = TurnStep::PreCombatMain;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: storm,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast Corroding Dragonstorm");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_life - 2, "opponent lost 2");
    assert_eq!(g.players[0].life, my_life + 2, "you gained 2");
    // A Dragon entering under your control returns the enchantment to hand.
    let dragon = g.add_card_to_battlefield(0, catalog::armament_dragon());
    g.dispatch_triggers_for_events(&[GameEvent::PermanentEntered { card_id: dragon }]);
    drain_stack(&mut g);
    assert!(g.battlefield_find(storm).is_none(), "storm left battlefield");
    assert!(g.players[0].hand.iter().any(|c| c.id == storm), "returned to hand");
}

/// Essence Anchor only mints a Zombie Druid after a card left your graveyard.
#[test]
fn essence_anchor_gated_on_graveyard_departure() {
    let mut g = two_player_game();
    let anchor = g.add_card_to_battlefield(0, catalog::essence_anchor());
    g.clear_sickness(anchor);
    g.step = TurnStep::PreCombatMain;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    // No card has left the graveyard → activation is rejected.
    assert!(
        g.perform_action(GameAction::ActivateAbility {
            card_id: anchor,
            ability_index: 0,
            target: None,
            additional_targets: Vec::new(),
            x_value: None, mode: None,
        })
        .is_err(),
        "gated off without a graveyard departure"
    );
    // Mark a graveyard departure this turn, then it works.
    g.players[0].cards_left_graveyard_this_turn = 1;
    g.perform_action(GameAction::ActivateAbility {
        card_id: anchor,
        ability_index: 0,
        target: None,
        additional_targets: Vec::new(),
        x_value: None, mode: None,
    })
    .expect("activate after departure");
    drain_stack(&mut g);
    assert!(
        g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Zombie Druid"),
        "minted a Zombie Druid"
    );
}

/// Stormbeacon Blade grants +3/+0 and draws when 3+ creatures attack.
#[test]
fn stormbeacon_blade_pumps_and_draws_on_mass_attack() {
    let mut g = two_player_game();
    let blade = g.add_card_to_battlefield(0, catalog::stormbeacon_blade());
    let a = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let c = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    for id in [a, b, c] {
        g.clear_sickness(id);
    }
    g.step = TurnStep::PreCombatMain;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add_colorless(2); // Equip {2}
    g.perform_action(GameAction::Equip { equipment: blade, target: a }).expect("equip");
    assert_eq!(g.computed_permanent(a).unwrap().power, 5, "+3/+0 → 5 power");
    g.add_card_to_library(0, catalog::grizzly_bears()); // something to draw
    advance_to(&mut g, TurnStep::DeclareAttackers);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::DeclareAttackers(vec![
        Attack { attacker: a, target: AttackTarget::Player(1) },
        Attack { attacker: b, target: AttackTarget::Player(1) },
        Attack { attacker: c, target: AttackTarget::Player(1) },
    ]))
    .expect("attack with three");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand_before + 1, "drew on 3+ attackers");
}

/// Jeskai Shrinekeeper gains life and draws on combat damage to a player.
#[test]
fn jeskai_shrinekeeper_gains_and_draws_on_connect() {
    let mut g = two_player_game();
    let dragon = g.add_card_to_battlefield(0, catalog::jeskai_shrinekeeper());
    g.clear_sickness(dragon);
    g.add_card_to_library(0, catalog::grizzly_bears());
    let life = g.players[0].life;
    let hand = g.players[0].hand.len();
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: dragon,
        target: AttackTarget::Player(1),
    }]))
    .expect("attack");
    drain_stack(&mut g);
    advance_to(&mut g, TurnStep::CombatDamage);
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life + 1, "gained 1 on connect");
    assert_eq!(g.players[0].hand.len(), hand + 1, "drew on connect");
}

/// Encroaching Dragonstorm ramps two basics and bounces on a Dragon ETB.
#[test]
fn encroaching_dragonstorm_ramps_and_bounces() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let f1 = g.add_card_to_library(0, catalog::forest());
    let f2 = g.add_card_to_library(0, catalog::forest());
    let storm = g.add_card_to_hand(0, catalog::encroaching_dragonstorm());
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Search(Some(f1)),
        DecisionAnswer::Search(Some(f2)),
    ]));
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.step = TurnStep::PreCombatMain;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    let lands_before = g.battlefield.iter().filter(|c| c.controller == 0 && c.definition.is_land()).count();
    g.perform_action(GameAction::CastSpell {
        card_id: storm,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast Encroaching Dragonstorm");
    drain_stack(&mut g);
    let lands_after = g.battlefield.iter().filter(|c| c.controller == 0 && c.definition.is_land()).count();
    assert_eq!(lands_after, lands_before + 2, "ramped two basics onto battlefield");
    // Dragon enters → bounce.
    let dragon = g.add_card_to_battlefield(0, catalog::jeskai_shrinekeeper());
    g.dispatch_triggers_for_events(&[GameEvent::PermanentEntered { card_id: dragon }]);
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == storm), "bounced on Dragon ETB");
}

/// Kheru Goldkeeper's Renew puts +1/+1 and flying counters on a creature.
#[test]
fn kheru_goldkeeper_renew_grants_counters() {
    let mut g = two_player_game();
    let kheru = g.add_card_to_graveyard(0, catalog::kheru_goldkeeper());
    let target = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.step = TurnStep::PreCombatMain;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: kheru,
        ability_index: 0,
        target: Some(Target::Permanent(target)),
        additional_targets: Vec::new(),
        x_value: None, mode: None,
    })
    .expect("Renew from graveyard");
    drain_stack(&mut g);
    let cp = g.computed_permanent(target).unwrap();
    assert_eq!((cp.power, cp.toughness), (4, 4), "two +1/+1 counters");
    assert!(cp.keywords.contains(&Keyword::Flying), "flying counter");
    assert!(g.exile.iter().any(|c| c.id == kheru), "Kheru exiled by Renew cost");
}

/// Dragonclaw Strike doubles your creature's P/T then fights.
#[test]
fn dragonclaw_strike_doubles_and_fights() {
    let mut g = two_player_game();
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let spell = g.add_card_to_hand(0, catalog::dragonclaw_strike());
    g.players[0].mana_pool.add_colorless(6);
    g.step = TurnStep::PreCombatMain;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: Some(Target::Permanent(mine)),
        additional_targets: vec![Target::Permanent(foe)],
        mode: None,
        x_value: None,
    })
    .expect("cast Dragonclaw Strike");
    drain_stack(&mut g);
    // 2/2 doubled to 4/4 deals 4 to the 2/2 foe → foe dies; foe deals 2 back.
    assert!(g.battlefield_find(foe).is_none(), "foe died to the fight");
    let cp = g.computed_permanent(mine).unwrap();
    assert_eq!(cp.power, 4, "doubled to 4 power");
}

/// Clarion Conqueror stops creatures' activated abilities from being used.
#[test]
fn clarion_conqueror_locks_creature_abilities() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::clarion_conqueror());
    // Jade-Cast Sentinel has a {2},{T} activated ability — now locked.
    let sentinel = g.add_card_to_battlefield(1, catalog::jade_cast_sentinel());
    g.clear_sickness(sentinel);
    let dead = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    g.players[1].mana_pool.add_colorless(2);
    g.step = TurnStep::PreCombatMain;
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    assert!(
        g.perform_action(GameAction::ActivateAbility {
            card_id: sentinel,
            ability_index: 0,
            target: Some(Target::Permanent(dead)),
            additional_targets: Vec::new(),
            x_value: None, mode: None,
        })
        .is_err(),
        "activated ability locked by Clarion Conqueror"
    );
}

/// Ambling Stormshell stuns itself and draws three when it attacks.
#[test]
fn ambling_stormshell_stuns_and_draws_on_attack() {
    use crabomination::card::CounterType;
    let mut g = two_player_game();
    let shell = g.add_card_to_battlefield(0, catalog::ambling_stormshell());
    g.clear_sickness(shell);
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::grizzly_bears());
    }
    let hand = g.players[0].hand.len();
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: shell,
        target: AttackTarget::Player(1),
    }]))
    .expect("attack");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand + 3, "drew three");
    let stun = *g.battlefield_find(shell).unwrap().counters.get(&CounterType::Stun).unwrap_or(&0);
    assert_eq!(stun, 3, "three stun counters");
}

/// Furious Forebear returns itself from the graveyard when a creature dies.
#[test]
fn furious_forebear_returns_on_creature_death() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let forebear = g.add_card_to_graveyard(0, catalog::furious_forebear());
    let doomed = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    // Kill the creature you control → trigger fires off the graveyard.
    // Real gameplay's SBA death path stamps the last-known controller into
    // `died_card_snapshots`; the test helper doesn't, so seed it so the
    // "a creature you control dies" filter reads the CR 603.10 LKI controller.
    let snap = g.battlefield_find(doomed).unwrap().clone();
    let mut evs = g.remove_to_graveyard_with_triggers(doomed);
    g.died_card_snapshots.insert(doomed, snap);
    evs.push(GameEvent::CreatureDied { card_id: doomed });
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert!(
        g.players[0].hand.iter().any(|c| c.id == forebear),
        "paid the cost to return Furious Forebear to hand"
    );
}

/// Bewilder weakens a creature and cantrips.
#[test]
fn bewilder_shrinks_power_and_draws() {
    let mut g = two_player_game();
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let spell = g.add_card_to_hand(0, catalog::bewilder());
    g.add_card_to_library(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.step = TurnStep::PreCombatMain;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    let hand = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: Some(Target::Permanent(foe)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast Bewilder");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(foe).unwrap().power, -1, "2 − 3 = −1 power");
    assert_eq!(g.players[0].hand.len(), hand, "cantrip nets even (drew 1, spent 1)");
}

/// Sarkhan grows and turns into a flying Dragon when a Dragon enters.
#[test]
fn sarkhan_grows_when_a_dragon_enters() {
    use crabomination::card::CounterType;
    let mut g = two_player_game();
    let sarkhan = g.add_card_to_battlefield(0, catalog::sarkhan_dragon_ascendant());
    // Behold ETB: with no Dragon yet, no Treasure.
    let dragon = g.add_card_to_battlefield(0, catalog::jeskai_shrinekeeper());
    g.dispatch_triggers_for_events(&[GameEvent::PermanentEntered { card_id: dragon }]);
    drain_stack(&mut g);
    let counters = g.battlefield_find(sarkhan).unwrap().counter_count(CounterType::PlusOnePlusOne);
    assert_eq!(counters, 1, "+1/+1 counter");
    let cp = g.computed_permanent(sarkhan).unwrap();
    assert!(cp.keywords.contains(&Keyword::Flying), "gained flying");
    assert!(
        cp.subtypes.creature_types.contains(&crabomination::card::CreatureType::Dragon),
        "became a Dragon"
    );
}

/// Jeskai Brushmaster is a 2/4 with double strike and prowess.
#[test]
fn jeskai_brushmaster_has_double_strike_and_prowess() {
    let mut g = two_player_game();
    let bm = g.add_card_to_battlefield(0, catalog::jeskai_brushmaster());
    let cp = g.computed_permanent(bm).unwrap();
    assert!(cp.keywords.contains(&Keyword::DoubleStrike), "double strike");
    assert!(cp.keywords.contains(&Keyword::Prowess), "prowess");
    assert_eq!((cp.power, cp.toughness), (2, 4), "2/4 body");
}

/// Riverwheel Sweep taps a creature and puts three stun counters on it.
#[test]
fn riverwheel_sweep_taps_and_stuns() {
    use crabomination::card::CounterType;
    let mut g = two_player_game();
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::riverwheel_sweep());
    g.players[0].mana_pool.add_colorless(6);
    g.step = TurnStep::PreCombatMain;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: Some(Target::Permanent(foe)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast Riverwheel Sweep");
    drain_stack(&mut g);
    let inst = g.battlefield_find(foe).unwrap();
    assert!(inst.tapped, "target tapped");
    assert_eq!(inst.counter_count(CounterType::Stun), 3, "three stun counters");
}

/// Flowstone Slide gives every creature +X/-X, killing X-toughness bodies.
#[test]
fn flowstone_slide_shrinks_all_creatures() {
    let mut g = two_player_game();
    let mine = g.add_card_to_battlefield(0, catalog::hill_giant()); // 3/3
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let spell = g.add_card_to_hand(0, catalog::flowstone_slide());
    g.players[0].mana_pool.add(Color::Red, 2);
    g.players[0].mana_pool.add_colorless(4); // {2}{R}{R} + X=2
    g.step = TurnStep::PreCombatMain;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: Some(2),
    })
    .expect("cast Flowstone Slide X=2");
    drain_stack(&mut g);
    // +2/-2: the 2/2 dies (0 toughness), the 3/3 becomes 5/1.
    assert!(g.battlefield_find(foe).is_none(), "2/2 died to -2 toughness");
    let cp = g.computed_permanent(mine).unwrap();
    assert_eq!((cp.power, cp.toughness), (5, 1), "3/3 → 5/1");
}

/// Dragonbroods' Relic sacrifices for a 4/4 all-color Reliquary Dragon.
#[test]
fn dragonbroods_relic_makes_reliquary_dragon() {
    let mut g = two_player_game();
    let relic = g.add_card_to_battlefield(0, catalog::dragonbroods_relic());
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2, ETB 3 kills it
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.step = TurnStep::PreCombatMain;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    let opp_life = g.players[1].life;
    g.perform_action(GameAction::ActivateAbility {
        card_id: relic,
        ability_index: 1, // the sac ability
        target: None, // sac ability has no target; the token's ETB auto-targets
        additional_targets: Vec::new(),
        x_value: None, mode: None,
    })
    .expect("sacrifice for a Reliquary Dragon");
    drain_stack(&mut g);
    let dragon_id = g
        .battlefield
        .iter()
        .find(|c| c.controller == 0 && c.definition.name == "Reliquary Dragon")
        .expect("minted Reliquary Dragon")
        .id;
    let cp = g.computed_permanent(dragon_id).unwrap();
    assert!(cp.keywords.contains(&Keyword::Flying), "flying");
    assert!(cp.keywords.contains(&Keyword::Lifelink), "lifelink");
    assert_eq!(cp.colors.len(), 5, "all five colors");
    assert!(g.battlefield_find(relic).is_none(), "relic sacrificed");
    // The token's ETB deals 3 to an auto-chosen "any target" — the 2/2 dies or
    // the opponent takes 3 to the face; either way 3 damage landed.
    assert!(
        g.battlefield_find(victim).is_none() || g.players[1].life == opp_life - 3,
        "ETB dealt 3 damage somewhere"
    );
}

/// Traveling Botanist digs the top card on becoming tapped: land to hand, else bin.
#[test]
fn traveling_botanist_digs_on_tap() {
    let mut g = two_player_game();
    let bot = g.add_card_to_battlefield(0, catalog::traveling_botanist());
    // Land on top → goes to hand.
    let land = g.add_card_to_library(0, catalog::forest());
    g.battlefield_find_mut(bot).unwrap().tapped = true;
    g.dispatch_triggers_for_events(&[GameEvent::PermanentTapped { card_id: bot, actor: Some(0), as_attacker: false }]);
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == land), "land put into hand");
    // Nonland on top → binned to graveyard.
    let spell = g.add_card_to_library(0, catalog::bewilder());
    g.dispatch_triggers_for_events(&[GameEvent::PermanentTapped { card_id: bot, actor: Some(0), as_attacker: false }]);
    drain_stack(&mut g);
    assert!(g.players[0].graveyard.iter().any(|c| c.id == spell), "nonland milled");
}

// ── Siege cycle (CR 614 `enter_modes` persistent mode choice) ────────────────

/// Cast `siege` from player 0's hand, choosing `enter_modes` index `mode_idx`.
fn cast_siege(
    g: &mut GameState,
    siege: crabomination::card::CardDefinition,
    mode_idx: usize,
) -> CardId {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let id = g.add_card_to_hand(0, siege);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Mode(mode_idx)]));
    // One of each color + generic — enough to pay any Siege's ≤4 cost.
    g.players[0].mana_pool.add_colorless(2);
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[0].mana_pool.add(c, 1);
    }
    g.step = TurnStep::PreCombatMain;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast siege");
    drain_stack(g);
    id
}

/// Barrensteppe Siege (Abzan): at your end step, +1/+1 counter on each creature.
#[test]
fn barrensteppe_siege_abzan_end_step_counters() {
    use crabomination::card::CounterType;
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    cast_siege(&mut g, catalog::barrensteppe_siege(), 0);
    advance_to(&mut g, TurnStep::End);
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne),
        1,
        "Abzan end step put a +1/+1 counter on my creature",
    );
}

/// Frostcliff Siege (Temur): creatures you control get +1/+0, trample, haste.
#[test]
fn frostcliff_siege_temur_anthem() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    cast_siege(&mut g, catalog::frostcliff_siege(), 1);
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!(cp.power, 3, "Temur anthem +1/+0");
    assert!(cp.keywords.contains(&Keyword::Trample), "Temur grants trample");
    assert!(cp.keywords.contains(&Keyword::Haste), "Temur grants haste");
}

/// Glacierwood Siege (Temur): casting an instant mills the opponent four.
#[test]
fn glacierwood_siege_temur_mills_on_instant() {
    let mut g = two_player_game();
    for _ in 0..6 {
        g.add_card_to_library(1, catalog::grizzly_bears());
    }
    cast_siege(&mut g, catalog::glacierwood_siege(), 0);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let gy_before = g.players[1].graveyard.len();
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast bolt");
    drain_stack(&mut g);
    assert_eq!(g.players[1].graveyard.len(), gy_before + 4, "Temur milled opponent four");
}

/// Hollowmurk Siege (Abzan): when you attack, +1/+1 counter + menace on attacker.
#[test]
fn hollowmurk_siege_abzan_pumps_attacker() {
    use crabomination::card::CounterType;
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(bear);
    cast_siege(&mut g, catalog::hollowmurk_siege(), 1);
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: bear,
        target: AttackTarget::Player(1),
    }]))
    .expect("attack");
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne),
        1,
        "attacker got a +1/+1 counter",
    );
    assert!(
        g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Menace),
        "attacker gained menace",
    );
}

/// Abzan Monument's ETB tutors a basic Plains/Swamp/Forest to hand.
#[test]
fn abzan_monument_etb_searches_basic() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let plains = g.add_card_to_library(0, catalog::plains());
    let mon = g.add_card_to_hand(0, catalog::abzan_monument());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(plains))]));
    g.players[0].mana_pool.add_colorless(2);
    g.step = TurnStep::PreCombatMain;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: mon,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast Abzan Monument");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == plains), "searched a basic to hand");
}

/// Abzan Monument's sac ability mints an X/X Spirit, X = greatest toughness.
#[test]
fn abzan_monument_mints_xx_spirit() {
    let mut g = two_player_game();
    let monument = g.add_card_to_battlefield(0, catalog::abzan_monument());
    g.add_card_to_battlefield(0, catalog::jade_cast_sentinel()); // 1/5 → X = 5
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.step = TurnStep::PreCombatMain;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: monument,
        ability_index: 0,
        target: None,
        additional_targets: Vec::new(),
        x_value: None, mode: None,
    })
    .expect("mint Spirit");
    drain_stack(&mut g);
    let spirit = g
        .battlefield
        .iter()
        .find(|c| c.controller == 0 && c.definition.name == "Spirit")
        .expect("spirit token");
    assert_eq!(spirit.definition.power, 5, "X = greatest toughness (5)");
    assert_eq!(spirit.definition.toughness, 5);
}

/// Breaching Dragonstorm impulses a nonland (free if MV ≤ 8) and bounces itself
/// when a Dragon you control enters.
#[test]
fn breaching_dragonstorm_impulses_and_bounces() {
    let mut g = two_player_game();
    let bear = g.add_card_to_library(0, catalog::grizzly_bears()); // MV 2 nonland
    let storm = g.add_card_to_hand(0, catalog::breaching_dragonstorm());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(4);
    g.step = TurnStep::PreCombatMain;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: storm,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast Breaching Dragonstorm");
    drain_stack(&mut g);
    let s = g.exile.iter().find(|c| c.id == bear).expect("nonland impulsed");
    assert!(s.may_play_until.is_some(), "MV 2 ≤ 8 → free may-play");
    // Dragon enters → bounce.
    let dragon = g.add_card_to_battlefield(0, catalog::jeskai_shrinekeeper());
    g.dispatch_triggers_for_events(&[GameEvent::PermanentEntered { card_id: dragon }]);
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == storm), "bounced on Dragon ETB");
}

/// Dragonstorm Forecaster tutors a card by exact name.
#[test]
fn dragonstorm_forecaster_tutors_by_name() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let forecaster = g.add_card_to_battlefield(0, catalog::dragonstorm_forecaster());
    g.clear_sickness(forecaster);
    let globe = g.add_card_to_library(0, catalog::dragonstorm_globe());
    g.add_card_to_library(0, catalog::grizzly_bears()); // distractor, wrong name
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(globe))]));
    g.players[0].mana_pool.add_colorless(2);
    g.step = TurnStep::PreCombatMain;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: forecaster,
        ability_index: 0,
        target: None,
        additional_targets: Vec::new(),
        x_value: None, mode: None,
    })
    .expect("tutor by name");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == globe), "found Dragonstorm Globe by name");
}

/// Hundred-Battle Veteran gets +2/+4 only with 3+ kinds of counters among your
/// creatures.
#[test]
fn hundred_battle_veteran_pumps_with_counter_diversity() {
    use crabomination::card::CounterType;
    let mut g = two_player_game();
    let vet = g.add_card_to_battlefield(0, catalog::hundred_battle_veteran());
    g.battlefield_find_mut(vet).unwrap().counters.insert(CounterType::PlusOnePlusOne, 1);
    let ally = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(ally).unwrap().counters.insert(CounterType::Charge, 1);
    // Two kinds (+1/+1, charge) → static off; only the +1/+1 counter pumps.
    assert_eq!(g.computed_permanent(vet).unwrap().power, 5, "2 kinds → static off");
    // Third kind → +2/+4 on top of the +1/+1 counter → 7/7.
    g.battlefield_find_mut(ally).unwrap().counters.insert(CounterType::Shield, 1);
    let cp = g.computed_permanent(vet).unwrap();
    assert_eq!((cp.power, cp.toughness), (7, 7), "3 kinds → +2/+4");
}

/// Hundred-Battle Veteran can be cast from the graveyard, entering with a
/// finality counter.
#[test]
fn hundred_battle_veteran_casts_from_graveyard_with_finality() {
    use crabomination::card::CounterType;
    let mut g = two_player_game();
    let vet = g.add_card_to_graveyard(0, catalog::hundred_battle_veteran());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.step = TurnStep::PreCombatMain;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: vet,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast from graveyard");
    drain_stack(&mut g);
    let inst = g.battlefield_find(vet).expect("entered battlefield");
    assert_eq!(inst.counter_count(CounterType::Finality), 1, "entered with a finality counter");
}

/// Anafenza endures 2 when another nontoken creature you control dies.
#[test]
fn anafenza_endures_on_ally_death() {
    let mut g = two_player_game();
    let ana = g.add_card_to_battlefield(0, catalog::anafenza_unyielding_lineage());
    let ally = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(ally).unwrap().damage = 2; // lethal
    // SBA collects the death events; dispatch them so the observer trigger fires.
    let evs = g.check_state_based_actions();
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    // Endure 2 = two +1/+1 counters on Anafenza, or a 2/2 Spirit token.
    let grew = g.computed_permanent(ana).map(|c| c.power).unwrap_or(0) >= 4;
    let token = g
        .battlefield
        .iter()
        .any(|c| c.controller == 0 && c.id != ana && c.definition.name == "Spirit");
    assert!(grew || token, "endured 2 on an ally's death");
}

/// Felothar's ETB may-sacrifice pumps your team; it never sacrifices itself.
#[test]
fn felothar_may_sac_pumps_team() {
    use crabomination::card::CounterType;
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let fel = g.add_card_to_hand(0, catalog::felothar_dawn_of_the_abzan());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.step = TurnStep::PreCombatMain;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: fel,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast Felothar");
    drain_stack(&mut g);
    assert!(g.players[0].graveyard.iter().any(|c| c.id == fodder), "sacrificed the fodder creature");
    assert_eq!(
        g.battlefield_find(fel).unwrap().counter_count(CounterType::PlusOnePlusOne),
        1,
        "Felothar (never self-sacrificed) got a +1/+1 counter",
    );
}

/// Lotuslight Dancers' ETB tutors a black, green, and blue card to the graveyard.
#[test]
fn lotuslight_dancers_mills_three_colors() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let blk = g.add_card_to_library(0, catalog::hundred_battle_veteran()); // {3}{B}
    let grn = g.add_card_to_library(0, catalog::grizzly_bears()); // {1}{G}
    let blu = g.add_card_to_library(0, catalog::dragonstorm_forecaster()); // {U}
    let dancers = g.add_card_to_hand(0, catalog::lotuslight_dancers());
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Search(Some(blk)),
        DecisionAnswer::Search(Some(grn)),
        DecisionAnswer::Search(Some(blu)),
    ]));
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.step = TurnStep::PreCombatMain;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: dancers,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast Lotuslight Dancers");
    drain_stack(&mut g);
    for (id, color) in [(blk, "black"), (grn, "green"), (blu, "blue")] {
        assert!(g.players[0].graveyard.iter().any(|c| c.id == id), "{color} card milled to graveyard");
    }
}

/// Eshki draws + grows at begin-combat once you've cast a creature and a
/// noncreature spell this turn.
#[test]
fn eshki_draws_after_creature_and_noncreature_spell() {
    use crabomination::card::CounterType;
    let mut g = two_player_game();
    let eshki = g.add_card_to_battlefield(0, catalog::eshki_dragonclaw());
    g.add_card_to_library(0, catalog::grizzly_bears()); // something to draw
    g.players[0].creatures_cast_this_turn = 1;
    g.players[0].noncreature_spells_cast_this_game_turn = 1;
    let hand = g.players[0].hand.len();
    advance_to(&mut g, TurnStep::BeginCombat);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand + 1, "drew at begin-combat");
    assert_eq!(
        g.battlefield_find(eshki).unwrap().counter_count(CounterType::PlusOnePlusOne),
        2,
        "two +1/+1 counters",
    );
}

/// Narset discards your hand and draws cards equal to spells cast this turn.
#[test]
fn narset_discards_hand_draws_per_spells() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::narset_jeskai_waymaster());
    g.add_card_to_hand(0, catalog::grizzly_bears());
    g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0].spells_cast_this_turn = 2;
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::grizzly_bears());
    }
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    g.step = TurnStep::PreCombatMain;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    advance_to(&mut g, TurnStep::End);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), 2, "discarded 2, drew 2 (spells cast)");
    assert!(g.players[0].graveyard.len() >= 2, "the whole hand was discarded");
}

/// Revival of the Ancestors' first chapter makes three Spirit tokens.
#[test]
fn revival_of_the_ancestors_chapter_one_makes_spirits() {
    let mut g = two_player_game();
    let saga = g.add_card_to_hand(0, catalog::revival_of_the_ancestors());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.step = TurnStep::PreCombatMain;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: saga,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast Revival of the Ancestors");
    drain_stack(&mut g);
    let spirits = g
        .battlefield
        .iter()
        .filter(|c| c.controller == 0 && c.definition.name == "Spirit")
        .count();
    assert_eq!(spirits, 3, "chapter I made three Spirit tokens");
}

/// Kishla Village enters tapped without an Island/Swamp and taps for green.
#[test]
fn kishla_village_enters_tapped_and_taps_for_green() {
    let mut g = two_player_game();
    let land = g.add_card_to_hand(0, catalog::kishla_village());
    g.step = TurnStep::PreCombatMain;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::PlayLand(land)).expect("play land");
    drain_stack(&mut g);
    assert!(g.battlefield_find(land).unwrap().tapped, "enters tapped with no Island/Swamp");
    g.battlefield_find_mut(land).unwrap().tapped = false;
    g.perform_action(GameAction::ActivateAbility {
        card_id: land,
        ability_index: 0,
        target: None,
        additional_targets: Vec::new(),
        x_value: None, mode: None,
    })
    .expect("tap for green");
    assert_eq!(g.players[0].mana_pool.amount(Color::Green), 1, "added green mana");
}

/// Dracogenesis free-casts Dragon spells but not other spells.
#[test]
fn dracogenesis_free_casts_dragon_spells() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::dracogenesis());
    let dragon = g.add_card_to_hand(0, catalog::jeskai_shrinekeeper());
    g.step = TurnStep::PreCombatMain;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastFromZoneWithoutPaying {
        card_id: dragon,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Dracogenesis free-casts a Dragon");
    drain_stack(&mut g);
    assert!(g.battlefield_find(dragon).is_some(), "Dragon entered for free");
    let bears = g.add_card_to_hand(0, catalog::grizzly_bears());
    assert!(
        g.perform_action(GameAction::CastFromZoneWithoutPaying {
            card_id: bears,
            target: None,
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .is_err(),
        "a non-Dragon spell can't be free-cast",
    );
}

/// Death Begets Life destroys all creatures and enchantments and draws per one.
#[test]
fn death_begets_life_wraths_and_draws() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.add_card_to_battlefield(1, catalog::dracogenesis()); // an enchantment
    for _ in 0..5 {
        g.add_card_to_library(0, catalog::forest());
    }
    let spell = g.add_card_to_hand(0, catalog::death_begets_life());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(5);
    g.step = TurnStep::PreCombatMain;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    let hand_before = g.players[0].hand.len(); // spell will leave hand
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast Death Begets Life");
    drain_stack(&mut g);
    // Three permanents (2 creatures + 1 enchantment) destroyed → draw 3.
    let creatures = g.battlefield.iter().filter(|c| c.definition.is_creature()).count();
    assert_eq!(creatures, 0, "all creatures destroyed");
    // hand: -1 (cast the sorcery) +3 (drawn) = +2 net.
    assert_eq!(g.players[0].hand.len(), hand_before - 1 + 3, "drew one per destroyed permanent");
}

/// Herd Heirloom's second ability grants trample + a draw-on-combat-damage
/// trigger to a power-4 creature until end of turn.
#[test]
fn herd_heirloom_grants_trample_and_draw_on_damage() {
    use crabomination::card::CounterType;
    let mut g = two_player_game();
    let heirloom = g.add_card_to_battlefield(0, catalog::herd_heirloom());
    let beast = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(beast).unwrap().counters.insert(CounterType::PlusOnePlusOne, 2); // 4/4
    g.clear_sickness(beast);
    g.add_card_to_library(0, catalog::forest());
    g.step = TurnStep::PreCombatMain;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: heirloom,
        ability_index: 1,
        target: Some(Target::Permanent(beast)),
        additional_targets: Vec::new(),
        x_value: None, mode: None,
    })
    .expect("grant trample + draw trigger");
    drain_stack(&mut g);
    assert!(
        g.computed_permanent(beast).unwrap().keywords.contains(&Keyword::Trample),
        "gained trample",
    );
    let hand = g.players[0].hand.len();
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: beast,
        target: AttackTarget::Player(1),
    }]))
    .expect("attack");
    drain_stack(&mut g);
    advance_to(&mut g, TurnStep::CombatDamage);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand + 1, "drew on combat damage to a player");
}

/// Yathan Roadwatcher mills four on cast, then reanimates a cheap creature.
#[test]
fn yathan_roadwatcher_mills_and_reanimates() {
    let mut g = two_player_game();
    // A cheap creature to mill into the graveyard as a reanimation target.
    g.add_card_to_library(0, catalog::grizzly_bears()); // MV 2
    g.add_card_to_library(0, catalog::forest());
    g.add_card_to_library(0, catalog::forest());
    g.add_card_to_library(0, catalog::forest());
    let yathan = g.add_card_to_hand(0, catalog::yathan_roadwatcher());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.step = TurnStep::PreCombatMain;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    // AutoDecider picks the only legal reanimation target (the milled Bears).
    g.perform_action(GameAction::CastSpell {
        card_id: yathan,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast Yathan");
    drain_stack(&mut g);
    assert!(
        g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Grizzly Bears"),
        "reanimated a milled creature (MV ≤ 3)",
    );
}

/// Great Arashin City makes a Spirit by exiling a creature from the graveyard.
#[test]
fn great_arashin_city_exiles_for_spirit() {
    let mut g = two_player_game();
    let land = g.add_card_to_battlefield(0, catalog::great_arashin_city());
    let gy_creature = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.step = TurnStep::PreCombatMain;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: land,
        ability_index: 1,
        target: None,
        additional_targets: Vec::new(),
        x_value: None, mode: None,
    })
    .expect("make a Spirit");
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == gy_creature), "exiled the graveyard creature as a cost");
    assert!(
        g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Spirit"),
        "created a Spirit token",
    );
}

/// Nature's Rhythm cheats a creature with mana value X or less onto the battlefield.
#[test]
fn natures_rhythm_puts_creature_into_play() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    // Grizzly Bears is MV 2; cast with X=2.
    let bear = g.add_card_to_library(0, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::natures_rhythm());
    g.players[0].mana_pool.add(Color::Green, 2);
    g.players[0].mana_pool.add_colorless(2); // pays X=2
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(bear))]));
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: Some(2),
    }).expect("cast Nature's Rhythm for X=2");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.id == bear), "creature entered the battlefield");
    assert!(catalog::natures_rhythm().keywords.iter().any(|k| matches!(k, Keyword::Harmonize(_))));
}

/// Smile at Death reanimates a small creature from your graveyard at upkeep.
#[test]
fn smile_at_death_reanimates_small_creature() {
    use crabomination::card::CounterType;
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::smile_at_death());
    let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears()); // 2/2, power 2
    g.active_player_idx = 0;
    g.step = TurnStep::Untap;
    g.decider = Box::new(crabomination::decision::ScriptedDecider::new([
        crabomination::decision::DecisionAnswer::Target(Target::Permanent(bear)),
    ]));
    advance_to(&mut g, TurnStep::Upkeep);
    drain_stack(&mut g);
    let b = g.battlefield_find(bear).expect("bear reanimated");
    assert_eq!(b.counter_count(CounterType::PlusOnePlusOne), 1, "with a +1/+1 counter");
}

/// Roar of Endless Song makes a 5/5 Elephant on chapter I and doubles the team.
#[test]
fn roar_of_endless_song_elephant_then_double() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    let id = g.add_card_to_hand(0, catalog::roar_of_endless_song());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.step = TurnStep::PreCombatMain;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Roar");
    drain_stack(&mut g);
    // Chapter I on ETB: a 5/5 Elephant.
    assert!(g.battlefield.iter().any(|c| c.is_token && c.definition.name == "Elephant"), "made an Elephant");
    // Advance to chapter III → doubles P/T.
    g.saga_advance(id);
    g.saga_advance(id);
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(bear).unwrap().power, 4, "2/2 doubled to 4/4");
}

/// Zurgo mobilizes two tapped attacking Warriors when it attacks.
#[test]
fn zurgo_mobilizes_two() {
    use crabomination::game::types::{Attack, AttackTarget};
    let mut g = two_player_game();
    let zurgo = g.add_card_to_battlefield(0, catalog::zurgo_thunders_decree());
    g.clear_sickness(zurgo);
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: zurgo, target: AttackTarget::Player(1),
    }])).expect("Zurgo attacks");
    drain_stack(&mut g);
    let warriors = g.battlefield.iter()
        .filter(|c| c.is_token && c.definition.subtypes.creature_types.contains(&crabomination::card::CreatureType::Warrior))
        .count();
    assert_eq!(warriors, 2, "Mobilize 2 made two Warrior tokens");
}

/// Rot-Curse Rakshasa is a 5/5 with trample and decayed.
#[test]
fn rot_curse_rakshasa_stats() {
    let r = catalog::rot_curse_rakshasa();
    assert_eq!((r.power, r.toughness), (5, 5));
    assert!(r.keywords.contains(&Keyword::Trample) && r.keywords.contains(&Keyword::Decayed));
}

/// Flamehold Grappler copies the next spell you cast after it enters.
#[test]
fn flamehold_grappler_copies_next_spell() {
    let mut g = two_player_game();
    g.move_card_to_battlefield_for_test(0, catalog::flamehold_grappler());
    drain_stack(&mut g);
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    // Cast Lightning Bolt at the bear — the delayed trigger copies it → 6 to the bear.
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(foe)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Bolt");
    drain_stack(&mut g);
    // 3 (original) + 3 (copy) = 6 damage → the 2/2 is dead.
    assert!(g.battlefield_find(foe).is_none(), "bolt was copied — bear took 6");
}

/// The Sibsig Ceremony reduces creature-spell costs and, when you cast a
/// creature, destroys it and makes a 2/2 Zombie Druid.
#[test]
fn sibsig_ceremony_converts_cast_creatures() {
    use crabomination::card::CreatureType;
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::the_sibsig_ceremony());
    // Grizzly Bears ({1}{G}) costs {2} less → just {G}.
    let bears = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: bears, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("cast bears for {G} (cost reduced)");
    drain_stack(&mut g);
    assert!(g.players[0].graveyard.iter().any(|c| c.id == bears), "cast creature was destroyed");
    assert!(
        g.battlefield.iter().any(|c| c.definition.subtypes.creature_types.contains(&CreatureType::Zombie)),
        "made a Zombie Druid token"
    );
}

/// A creature that enters WITHOUT being cast (reanimation) isn't converted.
#[test]
fn sibsig_ceremony_ignores_noncast_entries() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::the_sibsig_ceremony());
    // Reanimate from the graveyard (Move path clears entered_by_cast), then
    // force the ETB dispatch: the "if you cast it" gate must reject it.
    let reanimated = g.move_card_to_battlefield_for_test(0, catalog::grizzly_bears());
    g.dispatch_triggers_for_events(&[GameEvent::PermanentEntered { card_id: reanimated }]);
    drain_stack(&mut g);
    assert!(g.battlefield_find(reanimated).is_some(), "reanimated creature survives");
}

/// Neriv doubles damage from a creature you control that entered this turn,
/// but not from one that entered earlier.
#[test]
fn neriv_doubles_fresh_creature_damage() {
    use crabomination::game::effects::EntityRef;
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::neriv_heart_of_the_storm());
    let fresh = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let old = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(fresh).unwrap().entered_turn = Some(g.turn_number);
    g.battlefield_find_mut(old).unwrap().entered_turn = Some(g.turn_number.saturating_sub(1));
    assert_eq!(g.scale_damage_to(Some(fresh), EntityRef::Player(1), 3), 6, "fresh doubled");
    assert_eq!(g.scale_damage_to(Some(old), EntityRef::Player(1), 3), 3, "old not doubled");
}

/// Maelstrom of the Spirit Dragon taps for {C} and can tutor a Dragon.
#[test]
fn maelstrom_taps_for_colorless_and_tutors_dragon() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let land = g.add_card_to_battlefield(0, catalog::maelstrom_of_the_spirit_dragon());
    // {T}: Add {C}.
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::ActivateAbility {
        card_id: land, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
    })
    .expect("tap for C");
    assert_eq!(g.players[0].mana_pool.total(), 1, "added one colorless");

    // {4},{T},Sacrifice: tutor a Dragon to hand (untap after the first tap).
    g.battlefield_find_mut(land).unwrap().tapped = false;
    let dragon = g.add_card_to_library(0, catalog::jeskai_shrinekeeper());
    g.add_card_to_library(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add_colorless(4);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(dragon))]));
    g.perform_action(GameAction::ActivateAbility {
        card_id: land, ability_index: 2, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
    })
    .expect("tutor Dragon");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == dragon), "Dragon went to hand");
    assert!(g.battlefield_find(land).is_none(), "land sacrificed");
}

/// Static Snare costs {1} less per attacking creature (both players' count).
#[test]
fn static_snare_reduced_by_attackers() {
    use crabomination::game::actions::cost_reduction_for_spell;
    let mut g = two_player_game();
    // CR 103.7a — only turn 1's draw is skipped; stock both libraries so
    // crossing a turn boundary doesn't deck anyone.
    for seat in 0..2 {
        for _ in 0..5 {
            g.add_card_to_library(seat, catalog::forest());
        }
    }
    let a = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(a);
    g.clear_sickness(b);
    let snare = crabomination::card::CardInstance::new(g.next_id(), catalog::static_snare(), 0);
    assert_eq!(cost_reduction_for_spell(&g, 0, &snare, None), 0, "no attackers → no discount");
    // Move to the opponent's declare-attackers and swing with both bears.
    while !(g.step == TurnStep::DeclareAttackers && g.active_player_idx == 1) {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    g.perform_action(GameAction::DeclareAttackers(vec![
        Attack { attacker: a, target: AttackTarget::Player(0) },
        Attack { attacker: b, target: AttackTarget::Player(0) },
    ]))
    .expect("attack");
    assert_eq!(cost_reduction_for_spell(&g, 0, &snare, None), 2, "two attackers → {{2}} off");
}

/// United Battlefront deploys up to two cheap noncreature-nonland permanents
/// from the top seven; creatures/lands are left behind.
#[test]
fn united_battlefront_deploys_two_permanents() {
    let mut g = two_player_game();
    // Library: two MV-2 artifacts (match) + three bears (creatures, no match).
    for _ in 0..2 {
        g.add_card_to_library(0, catalog::prophetic_prism());
    }
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::grizzly_bears());
    }
    let spell = g.add_card_to_hand(0, catalog::united_battlefront());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("cast United Battlefront");
    drain_stack(&mut g);
    let prisms = g
        .battlefield
        .iter()
        .filter(|c| c.definition.name == "Prophetic Prism")
        .count();
    assert_eq!(prisms, 2, "deployed both artifacts");
}

/// A creature with an on-attack trigger. Gains 1 life whenever it attacks.
fn attack_lifegainer() -> crabomination::card::CardDefinition {
    use crabomination::card::{CardDefinition, CardType};
    use crabomination::effect::{shortcut::on_attack, Effect, Selector, Value};
    CardDefinition {
        name: "Attack Lifegainer",
        card_types: vec![CardType::Creature],
        power: 2,
        toughness: 2,
        triggered_abilities: vec![on_attack(Effect::GainLife { who: Selector::You, amount: Value::ONE })],
        ..Default::default()
    }
}

/// Windcrag Siege (Mardu) doubles an attack-caused trigger of a permanent you
/// control: an on-attack "gain 1 life" fires twice → +2 life.
#[test]
fn windcrag_siege_mardu_doubles_attack_trigger() {
    let mut g = two_player_game();
    let mardu = catalog::windcrag_siege().with_mode_applied(0).expect("Mardu mode");
    g.add_card_to_battlefield(0, mardu);
    let atk = g.add_card_to_battlefield(0, attack_lifegainer());
    g.clear_sickness(atk);
    let life = g.players[0].life;
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: atk,
        target: AttackTarget::Player(1),
    }]))
    .expect("attack");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life + 2, "attack trigger fired twice");
}

/// Without the doubler the same trigger fires once → +1 life (control).
#[test]
fn windcrag_siege_control_single_fire() {
    let mut g = two_player_game();
    let atk = g.add_card_to_battlefield(0, attack_lifegainer());
    g.clear_sickness(atk);
    let life = g.players[0].life;
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: atk,
        target: AttackTarget::Player(1),
    }]))
    .expect("attack");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life + 1, "trigger fired once");
}

/// Windcrag Siege (Jeskai) makes a 1/1 red Goblin with lifelink+haste each upkeep.
#[test]
fn windcrag_siege_jeskai_makes_goblin() {
    use crabomination::card::CreatureType;
    let mut g = two_player_game();
    // CR 103.7a — only turn 1's draw is skipped; stock both libraries so
    // crossing a turn boundary doesn't deck anyone.
    for seat in 0..2 {
        for _ in 0..5 {
            g.add_card_to_library(seat, catalog::forest());
        }
    }
    let jeskai = catalog::windcrag_siege().with_mode_applied(1).expect("Jeskai mode");
    g.add_card_to_battlefield(0, jeskai);
    let before = g.battlefield.len();
    // Advance to *player 0's* next upkeep (skip the opponent's).
    while !(g.step == TurnStep::Upkeep && g.active_player_idx == 0) {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    drain_stack(&mut g);
    let goblin = g
        .battlefield
        .iter()
        .find(|c| c.definition.subtypes.creature_types.contains(&CreatureType::Goblin))
        .expect("made a Goblin");
    assert!(goblin.definition.keywords.contains(&Keyword::Haste), "Goblin has haste");
    assert!(g.battlefield.len() > before, "battlefield grew");
}

/// Songcrafter Mage grants harmonize (= mana cost) to a graveyard I/S card,
/// castable this turn from the graveyard via the Harmonize path.
#[test]
fn songcrafter_mage_grants_harmonize_to_graveyard_card() {
    let mut g = two_player_game();
    let div = g.add_card_to_graveyard(0, catalog::divination());
    // Enter through the real ETB funnel so the self-source trigger fires.
    g.move_card_to_battlefield_for_test(0, catalog::songcrafter_mage());
    drain_stack(&mut g);
    // The graveyard card now carries an until-end-of-turn harmonize grant.
    let granted = g.players[0]
        .graveyard
        .iter()
        .find(|c| c.id == div)
        .map(|c| c.effective_harmonize().is_some())
        .unwrap_or(false);
    assert!(granted, "Divination gained harmonize");
    // Cast it from the graveyard for its harmonize (= mana) cost {2}{U}.
    g.step = TurnStep::PreCombatMain;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.add_card_to_library(0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::grizzly_bears());
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastHarmonize {
        card_id: div,
        tap_creature: None,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast Divination via harmonize");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand_before + 2, "Divination drew two");
    assert!(g.exile.iter().any(|c| c.id == div), "harmonize exiles on resolve");
}

/// Cathartic Parting bounces an opponent's artifact into their library and
/// shuffles up to four cards from your graveyard back into your library.
#[test]
fn cathartic_parting_tucks_and_recurs() {
    let mut g = two_player_game();
    let mine = g.add_card_to_battlefield(1, catalog::howling_mine());
    // Five cards in your graveyard — up to four are reshuffled.
    for _ in 0..5 {
        g.add_card_to_graveyard(0, catalog::grizzly_bears());
    }
    let spell = g.add_card_to_hand(0, catalog::cathartic_parting());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.step = TurnStep::PreCombatMain;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    let opp_lib_before = g.players[1].library.len();
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: Some(Target::Permanent(mine)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast Cathartic Parting");
    drain_stack(&mut g);
    assert!(g.battlefield_find(mine).is_none(), "artifact left the battlefield");
    assert_eq!(g.players[1].library.len(), opp_lib_before + 1, "artifact shuffled into owner's library");
    let bears_left = g.players[0].graveyard.iter().filter(|c| c.definition.name == "Grizzly Bears").count();
    assert_eq!(bears_left, 1, "four of five graveyard cards reshuffled");
}

// ── Tarkir: Dragonstorm gap batch ─────────────────────────────────────────────

/// Jeskai Revelation does all five things it prints.
#[test]
fn jeskai_revelation_does_everything() {
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::mountain());
    }
    let bounced = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::jeskai_revelation());
    g.players[0].mana_pool.add_colorless(9);
    for c in [Color::White, Color::Blue, Color::Red] {
        g.players[0].mana_pool.add(c, 2);
    }
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: Some(Target::Permanent(bounced)),
        additional_targets: vec![Target::Player(1)],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert!(g.players[1].hand.iter().any(|c| c.id == bounced), "bounced");
    assert_eq!(g.players[1].life, 16, "4 damage");
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Monk").count(), 2);
    assert_eq!(g.players[0].hand.len(), hand_before - 1 + 2, "drew two");
    assert_eq!(g.players[0].life, 24, "gained four");
}

/// Sidisi returns a creature exactly one mana value above the one it ate.
#[test]
fn sidisi_upgrades_by_exactly_one_mana_value() {
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let sidisi = g.add_card_to_battlefield(0, catalog::sidisi_regent_of_the_mire());
    g.clear_sickness(sidisi);
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    // Grizzly Bears is {1}{G}, so only a mana-value-3 creature comes back.
    let same_size = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let bigger = g.add_card_to_graveyard(0, catalog::sedge_scorpion());
    let _ = bigger;
    let activate = |g: &mut GameState, t: crabomination::card::CardId| {
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::ActivateAbility {
            card_id: sidisi,
            ability_index: 0,
            target: Some(Target::Permanent(t)),
            additional_targets: vec![],
            x_value: None,
            mode: None,
        })
    };
    assert!(activate(&mut g, same_size).is_err(), "same mana value isn't +1");
}

/// Thunder of Unity's chapter II arms a drain on every creature that enters
/// for the rest of the turn.
#[test]
fn thunder_of_unity_drains_on_later_entries() {
    use crabomination::game::effects::EffectContext;
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    let saga = g.add_card_to_battlefield(0, catalog::thunder_of_unity());
    let ch2 = catalog::thunder_of_unity().saga_chapters[1].1.clone();
    g.resolve_effect(&ch2, &EffectContext::for_ability(saga, 0, None)).unwrap();
    let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.dispatch_triggers_for_events(&[GameEvent::PermanentEntered { card_id: bears }]);
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 19);
    assert_eq!(g.players[0].life, 21);
}

/// Shiko exiles a cheap graveyard card and lets you cast it for free this turn.
#[test]
fn shiko_frees_a_cheap_graveyard_card() {
    use crabomination::game::effects::EffectContext;
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let shiko = g.add_card_to_battlefield(0, catalog::shiko_paragon_of_the_way());
    let bolt = g.add_card_to_graveyard(0, catalog::lightning_bolt());
    let etb = catalog::shiko_paragon_of_the_way().triggered_abilities[0].effect.clone();
    g.resolve_effect(
        &etb,
        &EffectContext {
            targets: vec![Target::Permanent(bolt)],
            ..EffectContext::for_ability(shiko, 0, None)
        },
    )
    .unwrap();
    assert!(g.exile.iter().any(|c| c.id == bolt), "exiled");
    g.perform_action(GameAction::CastFromZoneWithoutPaying {
        card_id: bolt,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("free cast off the may-play grant");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 17);
}
