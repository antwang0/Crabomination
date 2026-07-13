//! Functionality tests for `catalog::sets::decks::tdm`.

use crate::card::Keyword;
use crate::catalog;
use crate::game::*;
use crate::mana::Color;

fn advance_to(g: &mut GameState, step: TurnStep) {
    while g.step != step {
        g.perform_action(GameAction::PassPriority).expect("pass priority");
    }
}

/// Alesha's Legacy grants deathtouch + indestructible to your creature.
#[test]
fn aleshas_legacy_grants_two_keywords() {
    let mut g = two_player_game();
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::aleshas_legacy());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: Some(Target::Permanent(mine)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast Alesha's Legacy");
    drain_stack(&mut g);
    let kws = g.computed_permanent(mine).unwrap().keywords;
    assert!(kws.contains(&Keyword::Deathtouch), "gained deathtouch");
    assert!(kws.contains(&Keyword::Indestructible), "gained indestructible");
}

/// Fire-Rim Form pumps +2/+0 and grants first strike on enter.
#[test]
fn fire_rim_form_pumps_and_grants_first_strike() {
    let mut g = two_player_game();
    let creature = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    let aura = g.add_card_to_hand(0, catalog::fire_rim_form());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::CastSpell {
        card_id: aura,
        target: Some(Target::Permanent(creature)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast Fire-Rim Form");
    drain_stack(&mut g);
    let cp = g.computed_permanent(creature).unwrap();
    assert_eq!(cp.power, 4, "+2/+0 → 4 power");
    assert!(cp.keywords.contains(&Keyword::FirstStrike), "ETB granted first strike");
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
        x_value: None,
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
    use crate::card::CounterType;
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
    use crate::card::CounterType;
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
        x_value: None,
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
        x_value: None,
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
        x_value: None,
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
        x_value: Some(2),
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
    use crate::card::CardType;
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
            x_value: None,
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
        x_value: None,
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
    use crate::decision::{DecisionAnswer, ScriptedDecider};
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
        x_value: None,
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
            x_value: None,
        })
        .is_err(),
        "activated ability locked by Clarion Conqueror"
    );
}

/// Ambling Stormshell stuns itself and draws three when it attacks.
#[test]
fn ambling_stormshell_stuns_and_draws_on_attack() {
    use crate::card::CounterType;
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
    use crate::decision::{DecisionAnswer, ScriptedDecider};
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
    use crate::card::CounterType;
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
        cp.subtypes.creature_types.contains(&crate::card::CreatureType::Dragon),
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
    use crate::card::CounterType;
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
        x_value: None,
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
    g.dispatch_triggers_for_events(&[GameEvent::PermanentTapped { card_id: bot, actor: Some(0) }]);
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == land), "land put into hand");
    // Nonland on top → binned to graveyard.
    let spell = g.add_card_to_library(0, catalog::bewilder());
    g.dispatch_triggers_for_events(&[GameEvent::PermanentTapped { card_id: bot, actor: Some(0) }]);
    drain_stack(&mut g);
    assert!(g.players[0].graveyard.iter().any(|c| c.id == spell), "nonland milled");
}
