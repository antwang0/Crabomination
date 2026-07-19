#![allow(unused_imports)]
use crabomination::card::{CardType, CounterType};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::*;
use crabomination::TurnStep;
use crabomination::game::{drain_stack, two_player_game};
use crabomination::mana::Color;
#[allow(unused)]
use crate::Factory;

// ── Theros god-weapons (THS) ──────────────────────────────────────────────────

/// Spear of Heliod: {1}{W}{W}, {T}: Destroy target creature that dealt
/// damage to you this turn.
#[test]
fn spear_of_heliod_destroys_creature_that_damaged_you() {
    use crabomination::game::{Attack, AttackTarget};
    let mut g = two_player_game();
    let spear = g.add_card_to_battlefield(0, catalog::spear_of_heliod());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(bear);

    // Bob's bear attacks Alice and connects.
    g.active_player_idx = 1;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: bear, target: AttackTarget::Player(0),
    }])).expect("bear attacks P0");
    g.step = TurnStep::CombatDamage;
    let life_before = g.players[0].life;
    g.resolve_combat().expect("combat resolves");
    assert_eq!(g.players[0].life, life_before - 2, "bear dealt 2 combat damage");
    assert!(g.players[0].creatures_that_damaged_me_this_turn.contains(&bear),
        "bear recorded as having damaged P0");

    // Alice spears the bear that hit her.
    g.players[0].mana_pool.add(Color::White, 2);
    g.players[0].mana_pool.add_colorless(1);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: spear, ability_index: 0,
        target: Some(Target::Permanent(bear)), additional_targets: Vec::new(), x_value: None,
    }).expect("Spear activates on the attacker");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none(), "bear destroyed");
    assert!(g.players[1].graveyard.iter().any(|c| c.id == bear), "bear in graveyard");
}

/// Hammer of Purphoros: {1}{R}, Sacrifice a land: Create a 3/3 colorless
/// Golem (sorcery speed).
#[test]
fn hammer_of_purphoros_sacs_land_for_golem() {
    let mut g = two_player_game();
    let hammer = g.add_card_to_battlefield(0, catalog::hammer_of_purphoros());
    let land = g.add_card_to_battlefield(0, catalog::mountain());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: hammer, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("Hammer activates at sorcery speed");
    drain_stack(&mut g);
    assert!(g.battlefield_find(land).is_none(), "land sacrificed");
    let golem = g.battlefield.iter().find(|c| c.controller == 0 && c.definition.name == "Golem");
    assert!(golem.is_some(), "3/3 Golem created");
    let g3 = g.computed_permanent(golem.unwrap().id).unwrap();
    assert_eq!((g3.power, g3.toughness), (3, 3));
}

/// Whip of Erebos: {2}{B}{B}, {T}: Reanimate a creature with haste; exile it
/// at the next end step.
#[test]
fn whip_of_erebos_reanimates_with_haste_then_exiles() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    let whip = g.add_card_to_battlefield(0, catalog::whip_of_erebos());
    // Seed a bear in P0's graveyard.
    let bear = g.add_card_to_library(0, catalog::grizzly_bears());
    let pos = g.players[0].library.iter().position(|c| c.id == bear).unwrap();
    let card = g.players[0].library.remove(pos);
    g.players[0].graveyard.push(card);

    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(2);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: whip, ability_index: 0,
        target: Some(Target::Permanent(bear)), additional_targets: Vec::new(), x_value: None,
    }).expect("Whip reanimates the bear");
    drain_stack(&mut g);
    let c = g.battlefield_find(bear).expect("bear reanimated");
    assert_eq!(c.controller, 0);
    assert!(g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Haste),
        "reanimated creature has haste");

    // At the next end step it's exiled (not returned to the graveyard).
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none(), "bear left the battlefield");
    assert!(g.exile.iter().any(|c| c.id == bear), "bear exiled, not in graveyard");
}

// ── Fading / Vanishing (CR 702.32 / 702.62) ────────────────────────────────

/// Fading N: enters with N fade counters; each upkeep removes one, and when
/// none remain to remove the permanent is sacrificed.
#[test]
fn fading_ticks_down_then_sacrifices_when_empty() {
    let mut g = two_player_game();
    let nexus = g.add_card_to_hand(0, catalog::parallax_nexus());
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: nexus, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Parallax Nexus castable");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(nexus).unwrap().counter_count(CounterType::Fade), 5);

    g.active_player_idx = 0;
    // Five upkeeps drain the counters one at a time.
    for expected in [4u32, 3, 2, 1, 0] {
        g.process_fading_vanishing();
        assert_eq!(g.battlefield_find(nexus).unwrap().counter_count(CounterType::Fade), expected);
    }
    // The sixth upkeep finds no fade counter to remove → sacrifice.
    g.process_fading_vanishing();
    assert!(g.battlefield_find(nexus).is_none(), "sacrificed once it can't pay Fading");
    assert!(g.players[0].graveyard.iter().any(|c| c.id == nexus));
}

/// Vanishing N: enters with N time counters; each upkeep removes one, and it's
/// sacrificed the upkeep the last time counter is removed.
#[test]
fn vanishing_sacrifices_when_last_time_counter_removed() {
    use crabomination::card::{CreatureType, Keyword, Subtypes};
    let mut g = two_player_game();
    let def = crabomination::card::CardDefinition {
        name: "Test Vanishing Bear",
        cost: crabomination::mana::cost(&[crabomination::mana::generic(2)]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Bear], ..Default::default() },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Vanishing(2)],
        ..Default::default()
    };
    let bear = g.add_card_to_battlefield(0, def);
    // add_card_to_battlefield bypasses the ETB-counters pipeline; seed them.
    g.battlefield_find_mut(bear).unwrap().add_counters(CounterType::Time, 2);
    g.active_player_idx = 0;

    g.process_fading_vanishing();
    assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::Time), 1,
        "one time counter removed, still alive");
    g.process_fading_vanishing();
    assert!(g.battlefield_find(bear).is_none(), "sacrificed when the last time counter is removed");
    assert!(g.players[0].graveyard.iter().any(|c| c.id == bear));
}

/// Parallax Tide: {0} exiles a land; when the Tide fades out (Fading) the
/// exiled land returns tapped under its owner's control.
#[test]
fn parallax_tide_exiles_land_and_returns_it_tapped_when_it_fades() {
    let mut g = two_player_game();
    let tide = g.add_card_to_battlefield(0, catalog::parallax_tide());
    g.battlefield_find_mut(tide).unwrap().add_counters(CounterType::Fade, 1);
    let opp_land = g.add_card_to_battlefield(1, catalog::island());

    // {0}: Exile the opponent's land until the Tide leaves.
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: tide, ability_index: 0,
        target: Some(Target::Permanent(opp_land)), additional_targets: Vec::new(), x_value: None,
    }).expect("Parallax Tide exiles a land");
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == opp_land), "land exiled");
    assert!(g.battlefield_find(opp_land).is_none());

    // Fade it out: first upkeep removes the last fade counter, second sacrifices.
    g.active_player_idx = 0;
    g.process_fading_vanishing();
    g.process_fading_vanishing();
    assert!(g.battlefield_find(tide).is_none(), "Tide sacrificed by Fading");
    let returned = g.battlefield_find(opp_land).expect("land returned");
    assert_eq!(returned.controller, 1, "returns under owner's control");
    assert!(returned.tapped, "returns tapped");
}

/// CR 704.5k — the World rule: a second World permanent (any controller)
/// sends all but the newest to their owners' graveyards.
#[test]
fn world_rule_keeps_only_the_newest_world_permanent() {
    use crabomination::card::Supertype;
    let world = |name: &'static str| crabomination::card::CardDefinition {
        name,
        cost: crabomination::mana::cost(&[crabomination::mana::generic(2)]),
        supertypes: vec![Supertype::World],
        card_types: vec![CardType::Enchantment],
        ..Default::default()
    };
    let mut g = two_player_game();
    let old = g.add_card_to_battlefield(0, world("Old World"));
    g.check_state_based_actions();
    assert!(g.battlefield_find(old).is_some(), "lone World permanent survives");

    // A second World permanent (different controller) triggers the rule.
    let new = g.add_card_to_battlefield(1, world("New World"));
    g.check_state_based_actions();
    assert!(g.battlefield_find(new).is_some(), "newest World permanent is kept");
    assert!(g.battlefield_find(old).is_none(), "older World permanent dies");
    assert!(g.players[0].graveyard.iter().any(|c| c.id == old),
        "older World goes to its owner's graveyard");
}

// ── Adventure (CR 715) ───────────────────────────────────────────────────────

/// Stomp (Bonecrusher Giant's adventure) deals 2 damage and exiles the card,
/// Queen of Ice's Rage of Winter taps a creature and stuns it (Stun counter),
/// so it stays tapped through its controller's next untap step.
#[test]
fn adventure_queen_of_ice_rage_taps_and_stuns() {
    use crabomination::card::CounterType;
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let qoi = g.add_card_to_hand(0, catalog::queen_of_ice());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastAdventure {
        card_id: qoi, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Rage of Winter");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).unwrap().tapped, "bear tapped");
    assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::Stun), 1, "stunned");
    // Controller's untap step removes the Stun counter instead of untapping.
    g.active_player_idx = 1;
    g.do_untap();
    assert!(g.battlefield_find(bear).unwrap().tapped, "still tapped (stun consumed)");
    assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::Stun), 0, "stun gone");
}

/// which can then be cast as the creature half from exile.
#[test]
fn adventure_bonecrusher_stomp_then_cast_creature() {
    let mut g = two_player_game();
    let wall = g.add_card_to_battlefield(1, catalog::wall_of_stone()); // 0/8, survives 2
    let id = g.add_card_to_hand(0, catalog::bonecrusher_giant());
    // Cast the Stomp half ({1}{R}).
    for _c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] { g.players[0].mana_pool.add(_c, 20); }
    g.players[0].mana_pool.add_colorless(20);
    g.perform_action(GameAction::CastAdventure {
        card_id: id, target: Some(Target::Permanent(wall)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Stomp");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(wall).unwrap().damage, 2, "Stomp deals 2");
    // The card now sits in exile, adventuring complete.
    assert!(g.exile.iter().any(|c| c.id == id && c.on_adventure), "exiled on adventure");
    // Cast the creature half from exile for {2}{R}.
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastAdventureCreature {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Bonecrusher Giant");
    drain_stack(&mut g);
    let giant = g.battlefield_find(id).expect("Giant on battlefield");
    assert_eq!((giant.power(), giant.toughness()), (4, 3));
    assert!(!giant.on_adventure && !giant.adventuring, "flags cleared once on battlefield");
}

/// Petty Theft (Brazen Borrower) returns an opponent's nonland permanent to
/// hand; the card is then castable as the 3/1 flier from exile.
#[test]
fn adventure_brazen_borrower_petty_theft_bounces() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::brazen_borrower());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastAdventure {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Petty Theft");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none(), "bear bounced");
    assert!(g.players[1].hand.iter().any(|c| c.id == bear), "bear back in owner's hand");
    assert!(g.exile.iter().any(|c| c.id == id && c.on_adventure), "Borrower exiled on adventure");
}

/// Swift End (Murderous Rider) destroys a creature and costs 2 life.
#[test]
fn adventure_murderous_rider_swift_end() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::murderous_rider());
    let life = g.players[0].life;
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastAdventure {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Swift End");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none(), "creature destroyed");
    assert_eq!(g.players[0].life, life - 2, "lose 2 life");
}

/// Profane Insight (Foulmire Knight) is an instant/sorcery cast, so it ticks
/// a Prowess counter on a creature you control.
#[test]
fn adventure_counts_as_noncreature_spell_for_prowess() {
    let mut g = two_player_game();
    let prowler = g.add_card_to_battlefield(0, catalog::monastery_swiftspear());
    let lib_id = g.next_id();
    g.players[0].library.push(CardInstance::new(lib_id, catalog::grizzly_bears(), 0));
    let id = g.add_card_to_hand(0, catalog::foulmire_knight());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    let life = g.players[0].life;
    g.perform_action(GameAction::CastAdventure {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Profane Insight");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life - 1, "Profane Insight: lose 1 life");
    let sw = g.battlefield_find(prowler).expect("swiftspear");
    assert_eq!(sw.power(), 2, "prowess fired on the adventure spell");
}

/// Alter Fate (Order of Midnight) returns a creature card from a graveyard
/// to its owner's hand.
#[test]
fn adventure_order_of_midnight_alter_fate_reanimates() {
    let mut g = two_player_game();
    let dead = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::order_of_midnight());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastAdventure {
        card_id: id, target: Some(Target::Permanent(dead)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Alter Fate");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == dead), "creature back in hand");
}

// ── Plot (CR 702.170) ────────────────────────────────────────────────────────

/// A plotted card is exiled face-up, can't be cast the turn it was plotted,
/// and casts for free (no mana) on a later turn.
#[test]
fn plot_spinewoods_paladin_casts_free_later() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::spinewoods_paladin());
    // Plot for {2}{G} — exactly enough so the pool is empty afterward.
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::Plot { card_id: id }).expect("plot");
    assert!(g.exile.iter().any(|c| c.id == id), "plotted card in exile");
    assert!(g.plotted_cards.contains(&id), "tracked as plotted");
    // Can't cast it the same turn (CR 702.170d).
    assert!(g.perform_action(GameAction::CastPlotted {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).is_err(), "can't cast the turn it was plotted");
    // On a later turn, cast for free (no mana in pool).
    g.plotted_this_turn.clear();
    assert!(g.players[0].mana_pool.total() == 0, "no mana available");
    g.perform_action(GameAction::CastPlotted {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast plotted for free");
    drain_stack(&mut g);
    let p = g.battlefield_find(id).expect("Paladin on battlefield");
    assert_eq!((p.power(), p.toughness()), (5, 4));
    assert!(!g.plotted_cards.contains(&id), "plot state cleared once cast");
}

/// Vault Plunderer's ETB still fires when it's cast from a plot.
#[test]
fn plot_vault_plunderer_etb_draw_on_free_cast() {
    let mut g = two_player_game();
    let lib_id = g.next_id();
    g.players[0].library.push(CardInstance::new(lib_id, catalog::grizzly_bears(), 0));
    let id = g.add_card_to_hand(0, catalog::vault_plunderer());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::Plot { card_id: id }).expect("plot");
    g.plotted_this_turn.clear();
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastPlotted {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast plotted");
    drain_stack(&mut g);
    assert!(g.battlefield_find(id).is_some(), "Plunderer entered");
    assert_eq!(g.players[0].hand.len(), hand_before + 1, "ETB drew a card");
}

// ── Saddle (CR 702.171) ──────────────────────────────────────────────────────

/// Saddling taps the saddlers and marks the Mount saddled; the attack trigger
/// only fires once the Mount is saddled.
#[test]
fn saddle_stingerback_attack_trigger_gated_on_saddled() {
    let mut g = two_player_game();
    let terror = g.add_card_to_battlefield(0, catalog::stingerback_terror());
    g.clear_sickness(terror);
    let b1 = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // power 2
    let b2 = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // power 2 -> total 4 >= 3
    g.priority.player_with_priority = 0;
    g.active_player_idx = 0;
    // Saddle 3: tap two 2-power bears.
    g.perform_action(GameAction::Saddle {
        mount: terror, creatures: vec![b1, b2],
    }).expect("saddle");
    assert!(g.battlefield_find(terror).unwrap().saddled, "Mount is saddled");
    assert!(g.battlefield_find(b1).unwrap().tapped && g.battlefield_find(b2).unwrap().tapped,
        "saddlers tapped");
    // Attack while saddled — each opponent loses half their life, rounded up.
    let life = g.players[1].life;
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: terror, target: AttackTarget::Player(1),
    }])).unwrap();
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life - (life + 1) / 2,
        "saddled attack drains half the opponent's life, rounded up");
}

/// Half-life drain rounds up on an odd total (7 → loses 4).
#[test]
fn stingerback_half_life_rounds_up() {
    let mut g = two_player_game();
    let terror = g.add_card_to_battlefield(0, catalog::stingerback_terror());
    g.clear_sickness(terror);
    g.battlefield_find_mut(terror).unwrap().saddled = true;
    g.players[1].life = 7;
    g.priority.player_with_priority = 0;
    g.active_player_idx = 0;
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: terror, target: AttackTarget::Player(1),
    }])).unwrap();
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 3, "7 → loses 4 (rounded up)");
}

/// Without saddling, the attack trigger does not fire.
#[test]
fn saddle_unsaddled_attack_no_trigger() {
    let mut g = two_player_game();
    let terror = g.add_card_to_battlefield(0, catalog::stingerback_terror());
    g.clear_sickness(terror);
    g.priority.player_with_priority = 0;
    g.active_player_idx = 0;
    let life = g.players[1].life;
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: terror, target: AttackTarget::Player(1),
    }])).unwrap();
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life, "no saddle, no drain trigger");
}

/// Saddle is rejected when the tapped creatures' total power is below N.
#[test]
fn saddle_insufficient_power_rejected() {
    let mut g = two_player_game();
    let terror = g.add_card_to_battlefield(0, catalog::stingerback_terror());
    let weenie = g.add_card_to_battlefield(0, catalog::elvish_mystic()); // power 1 < 3
    g.priority.player_with_priority = 0;
    g.active_player_idx = 0;
    assert!(g.perform_action(GameAction::Saddle {
        mount: terror, creatures: vec![weenie],
    }).is_err(), "1 power can't pay Saddle 3");
    assert!(!g.battlefield_find(terror).unwrap().saddled, "not saddled");
}

// ── Casualty (CR 702.153) ────────────────────────────────────────────────────

/// Casualty: paying the cost (sacrificing a creature) copies the spell, so
/// Cut of the Profits resolves twice — double draw and double life loss.
#[test]
fn casualty_cut_of_the_profits_copies_spell() {
    let mut g = two_player_game();
    let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // power 2 >= 1
    for _ in 0..6 { g.add_card_to_library(0, catalog::mountain()); }
    let id = g.add_card_to_hand(0, catalog::cut_of_the_profits());
    // {X=2}{B}{B}.
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(2);
    let life = g.players[0].life;
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpellCasualty {
        card_id: id, sacrifice: fodder, target: None,
        additional_targets: vec![], mode: None, x_value: Some(2),
    }).expect("cast with casualty");
    drain_stack(&mut g);
    assert!(g.battlefield_find(fodder).is_none(), "casualty creature sacrificed");
    // Original + copy each draw 2 / lose 2.
    assert_eq!(g.players[0].hand.len(), hand_before - 1 + 4, "drew 2 + 2 from the copy");
    assert_eq!(g.players[0].life, life - 4, "lost 2 + 2 from the copy");
}

/// Casualty is optional: a normal cast (no sacrifice) makes no copy.
#[test]
fn casualty_normal_cast_no_copy() {
    let mut g = two_player_game();
    for _ in 0..4 { g.add_card_to_library(0, catalog::mountain()); }
    let id = g.add_card_to_hand(0, catalog::cut_of_the_profits());
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(2);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: Some(2),
    }).expect("plain cast");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand_before - 1 + 2, "drew 2, no copy");
}

// ── More Adventure cards (CR 715) ────────────────────────────────────────────

/// Rider in Need (Lonesome Unicorn) makes a 2/2 Knight; the Unicorn casts later.
#[test]
fn adventure_lonesome_unicorn_rider_in_need() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::lonesome_unicorn());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastAdventure {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Rider in Need");
    drain_stack(&mut g);
    assert_eq!(g.battlefield.iter().filter(|c| c.controller == 0
        && c.definition.name == "Knight").count(), 1, "made a Knight");
    assert!(g.exile.iter().any(|c| c.id == id && c.on_adventure));
}

/// Harvest Fear (Reaper of Night) makes the opponent discard two.
#[test]
fn adventure_reaper_of_night_harvest_fear() {
    let mut g = two_player_game();
    g.add_card_to_hand(1, catalog::lightning_bolt());
    g.add_card_to_hand(1, catalog::shock());
    let id = g.add_card_to_hand(0, catalog::reaper_of_night());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastAdventure {
        card_id: id, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Harvest Fear");
    drain_stack(&mut g);
    assert_eq!(g.players[1].hand.len(), 0, "opponent discarded both cards");
}

/// Cast Off (Realm-Cloaked Giant) destroys non-Giants but spares Giants.
#[test]
fn adventure_realm_cloaked_giant_cast_off_spares_giants() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let giant = g.add_card_to_battlefield(1, catalog::charging_monstrosaur()); // not a Giant
    let friendly_giant = g.add_card_to_battlefield(0, catalog::bonecrusher_giant());
    let id = g.add_card_to_hand(0, catalog::realm_cloaked_giant());
    g.players[0].mana_pool.add(Color::White, 2);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastAdventure {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Cast Off");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none(), "non-Giant destroyed");
    assert!(g.battlefield_find(giant).is_none(), "Dinosaur (non-Giant) destroyed");
    assert!(g.battlefield_find(friendly_giant).is_some(), "Giant spared");
}

/// Welcome Home (Flaxen Intruder) makes three 2/2 Bears.
#[test]
fn adventure_flaxen_intruder_welcome_home() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::flaxen_intruder());
    g.players[0].mana_pool.add(Color::Green, 3);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastAdventure {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Welcome Home");
    drain_stack(&mut g);
    assert_eq!(g.battlefield.iter().filter(|c| c.controller == 0
        && c.definition.name == "Bear").count(), 3, "three Bears");
}

/// Usher to Safety (Shepherd of the Flock) bounces your own permanent.
#[test]
fn adventure_shepherd_usher_to_safety_bounces_own() {
    let mut g = two_player_game();
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::shepherd_of_the_flock());
    g.players[0].mana_pool.add(Color::White, 1);
    g.perform_action(GameAction::CastAdventure {
        card_id: id, target: Some(Target::Permanent(mine)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Usher to Safety");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == mine), "own permanent back in hand");
}

/// Haggle (Merchant of the Vale) draws then discards (loots).
#[test]
fn adventure_merchant_of_the_vale_haggle_loots() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::mountain());
    g.add_card_to_hand(0, catalog::shock()); // a card to discard
    let id = g.add_card_to_hand(0, catalog::merchant_of_the_vale());
    g.players[0].mana_pool.add(Color::Red, 1);
    let hand_before = g.players[0].hand.len(); // includes Merchant + Shock
    g.perform_action(GameAction::CastAdventure {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Haggle");
    drain_stack(&mut g);
    // -1 (Merchant leaves) +1 (draw) -1 (discard) = hand_before - 1.
    assert_eq!(g.players[0].hand.len(), hand_before - 1, "drew then discarded");
}

// ── Affordance hints for new mechanics ───────────────────────────────────────

/// The hand-affordance sweep surfaces plottable and adventurable cards.
#[test]
fn affordances_surface_plot_and_adventure() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(1, catalog::grizzly_bears()); // a target for Stomp
    let plot = g.add_card_to_hand(0, catalog::spinewoods_paladin());
    let adv = g.add_card_to_hand(0, catalog::bonecrusher_giant());
    // Enough mana for both the plot cost ({2}{G}) and Stomp ({1}{R}).
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.priority.player_with_priority = 0;
    g.active_player_idx = 0;
    let a = g.compute_hand_affordances(0);
    assert!(a.plottable.contains(&plot), "Spinewoods Paladin is plottable");
    assert!(a.adventurable.contains(&adv), "Bonecrusher Giant's Stomp is castable");
}

/// Oaken Boon (Tuinvale Treefolk) puts two +1/+1 counters on a creature.
#[test]
fn adventure_tuinvale_oaken_boon_counters() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::tuinvale_treefolk());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastAdventure {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Oaken Boon");
    drain_stack(&mut g);
    let b = g.battlefield_find(bear).unwrap();
    assert_eq!((b.power(), b.toughness()), (4, 4), "+2/+2 from two counters");
}

/// Treats to Share (Curious Pair) makes a Food token.
#[test]
fn adventure_curious_pair_treats_to_share_food() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::curious_pair());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::CastAdventure {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Treats to Share");
    drain_stack(&mut g);
    assert_eq!(g.battlefield.iter().filter(|c| c.controller == 0
        && c.definition.name == "Food").count(), 1, "made a Food token");
}

/// Rage of Winter (Queen of Ice) taps a creature.
#[test]
fn adventure_queen_of_ice_rage_of_winter_taps() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::queen_of_ice());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastAdventure {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Rage of Winter");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).unwrap().tapped, "creature tapped");
}

/// Mesmeric Glare (Hypnotic Sprite) counters a cheap spell on the stack.
#[test]
fn adventure_hypnotic_sprite_mesmeric_glare_counters() {
    let mut g = two_player_game();
    // Opponent casts Lightning Bolt (MV 1) targeting our face.
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(0)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("opp casts Bolt");
    // We respond with Mesmeric Glare.
    let id = g.add_card_to_hand(0, catalog::hypnotic_sprite());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastAdventure {
        card_id: id, target: Some(Target::Permanent(bolt)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Mesmeric Glare");
    drain_stack(&mut g);
    assert!(g.players[1].graveyard.iter().any(|c| c.id == bolt), "Bolt countered to graveyard");
    assert_eq!(g.players[0].life, 20, "Bolt countered, no damage");
}

/// Heart's Desire (Lovestruck Beast) makes a 1/1 white Human token.
#[test]
fn adventure_lovestruck_beast_hearts_desire() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::lovestruck_beast());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::CastAdventure {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Heart's Desire");
    drain_stack(&mut g);
    assert_eq!(g.battlefield.iter().filter(|c| c.controller == 0
        && c.definition.name == "Human").count(), 1, "made a Human token");
}

/// Slickshot Show-Off pumps +2/+0 and draws when you cast a noncreature spell;
/// it can also be plotted for {R}.
#[test]
fn slickshot_show_off_pumps_and_draws_on_noncreature() {
    let mut g = two_player_game();
    let slick = g.add_card_to_battlefield(0, catalog::slickshot_show_off());
    g.add_card_to_library(0, catalog::mountain());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let hand_before = g.players[0].hand.len();
    cast_at(&mut g, bolt, Target::Player(1));
    let s = g.battlefield_find(slick).expect("slickshot");
    assert_eq!(s.power(), 3, "+2/+0 from the noncreature cast");
    // -1 (Bolt leaves) +1 (draw) = same hand size.
    assert_eq!(g.players[0].hand.len(), hand_before - 1 + 1, "drew a card");
}

/// Outcaster Trailblazer draws + makes a Treasure when you cast a 5+ MV spell,
/// and can be plotted for {2}{G}.
#[test]
fn outcaster_trailblazer_pays_off_big_spells() {
    let mut g = two_player_game();
    let trail = g.add_card_to_battlefield(0, catalog::outcaster_trailblazer());
    let _ = trail;
    g.add_card_to_library(0, catalog::mountain());
    // A 5-MV spell: Carnage Tyrant is {4}{G}{G} (MV 6).
    let big = g.add_card_to_hand(0, catalog::charging_monstrosaur()); // {3}{R}{R} MV5
    g.players[0].mana_pool.add(Color::Red, 2);
    g.players[0].mana_pool.add_colorless(3);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: big, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast a 5-drop");
    drain_stack(&mut g);
    // -1 (the 5-drop leaves hand) +1 (draw) = net same.
    assert_eq!(g.players[0].hand.len(), hand_before - 1 + 1, "drew off the big spell");
    assert_eq!(g.battlefield.iter().filter(|c| c.controller == 0
        && c.definition.name == "Treasure").count(), 1, "made a Treasure");
}

// ── Burn & utility additions ─────────────────────────────────────────────────

/// Lightning Helix deals 3 and gains 3 life.
#[test]
fn lightning_helix_burns_and_gains() {
    let mut g = two_player_game();
    let life = g.players[0].life;
    let id = g.add_card_to_hand(0, catalog::lightning_helix());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    cast_at(&mut g, id, Target::Player(1));
    assert_eq!(g.players[1].life, 17, "3 damage to the opponent");
    assert_eq!(g.players[0].life, life + 3, "gained 3 life");
}

/// Pillar of Flame exiles a creature it kills instead of letting it die.
#[test]
fn pillar_of_flame_exiles_what_it_kills() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let id = g.add_card_to_hand(0, catalog::pillar_of_flame());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    cast_at(&mut g, id, Target::Permanent(bear));
    assert!(g.battlefield_find(bear).is_none(), "creature killed");
    assert!(!g.players[1].graveyard.iter().any(|c| c.id == bear), "not in graveyard");
    assert!(g.exile.iter().any(|c| c.id == bear), "exiled instead");
}

/// Venture Deeper (Merfolk Secretkeeper) mills the opponent four.
#[test]
fn adventure_merfolk_secretkeeper_venture_deeper_mills() {
    let mut g = two_player_game();
    for _ in 0..6 { g.add_card_to_library(1, catalog::mountain()); }
    let before = g.players[1].library.len();
    let id = g.add_card_to_hand(0, catalog::merfolk_secretkeeper());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.perform_action(GameAction::CastAdventure {
        card_id: id, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Venture Deeper");
    drain_stack(&mut g);
    assert_eq!(g.players[1].library.len(), before - 4, "milled four");
}

/// Goblin Guide is a {R} 2/2 with haste.
#[test]
fn goblin_guide_is_hasty_two_two() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::goblin_guide());
    let c = g.battlefield_find(id).unwrap();
    assert_eq!((c.power(), c.toughness()), (2, 2));
    assert!(c.definition.keywords.contains(&Keyword::Haste));
}

/// Stitcher's Supplier mills three on ETB and three more on death.
#[test]
fn stitchers_supplier_mills_on_etb_and_death() {
    let mut g = two_player_game();
    for _ in 0..10 { g.add_card_to_library(0, catalog::mountain()); }
    let before = g.players[0].library.len();
    let id = g.add_card_to_hand(0, catalog::stitchers_supplier());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.players[0].library.len(), before - 3, "milled 3 on ETB");
    // Kill it.
    g.remove_to_graveyard_with_triggers(id);
    drain_stack(&mut g);
    assert_eq!(g.players[0].library.len(), before - 6, "milled 3 more on death");
}

/// Monastery Mentor makes a 1/1 Monk (with prowess) when you cast a
/// noncreature spell.
#[test]
fn monastery_mentor_spawns_monks() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::monastery_mentor());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    cast_at(&mut g, bolt, Target::Player(1));
    let monks: Vec<_> = g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.definition.name == "Monk").collect();
    assert_eq!(monks.len(), 1, "made one Monk token");
    assert!(!monks[0].definition.triggered_abilities.is_empty(), "Monk has prowess");
}

/// Spark Elemental is a 3/1 trample/haste that sacrifices itself at end step.
#[test]
fn spark_elemental_sacrifices_at_end_step() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::spark_elemental());
    let c = g.battlefield_find(id).unwrap();
    assert_eq!((c.power(), c.toughness()), (3, 1));
    assert!(c.definition.keywords.contains(&Keyword::Trample) &&
        c.definition.keywords.contains(&Keyword::Haste));
    // Walk to the end step on the controller's turn.
    g.active_player_idx = 0;
    g.step = TurnStep::End;
    g.priority.player_with_priority = 0;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert!(g.battlefield_find(id).is_none(), "sacrificed at end step");
    assert!(g.players[0].graveyard.iter().any(|c| c.id == id));
}

/// Keldon Marauders pings the opponent on entry and again when it leaves
/// (Vanishing 2 ticks it off).
#[test]
fn keldon_marauders_pings_on_etb_and_death() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::keldon_marauders());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 19, "ETB ping");
    g.remove_to_graveyard_with_triggers(id);
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 18, "leaves-play ping");
}

/// Ball Lightning is a 6/1 trample/haste that also self-sacrifices at end step.
#[test]
fn ball_lightning_is_six_one_and_self_sacrifices() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::ball_lightning());
    assert_eq!((g.battlefield_find(id).unwrap().power(),
        g.battlefield_find(id).unwrap().toughness()), (6, 1));
    g.active_player_idx = 0;
    g.step = TurnStep::End;
    g.priority.player_with_priority = 0;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert!(g.battlefield_find(id).is_none(), "sacrificed at end step");
}

/// Hellspark Elemental can be recast from the graveyard via Flashback.
#[test]
fn hellspark_elemental_has_flashback() {
    let mut g = two_player_game();
    let id = g.add_card_to_graveyard(0, catalog::hellspark_elemental());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastFlashback {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("flashback cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(id).is_some(), "Hellspark on the battlefield via flashback");
}

/// Isamaru is a {W} legendary 2/2.
#[test]
fn isamaru_is_legendary_two_two() {
    use crabomination::card::Supertype;
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::isamaru_hound_of_konda());
    let c = g.battlefield_find(id).unwrap();
    assert_eq!((c.power(), c.toughness()), (2, 2));
    assert!(c.definition.supertypes.contains(&Supertype::Legendary));
}

/// Mogg War Marshal makes a Goblin on entry and another on death.
#[test]
fn mogg_war_marshal_makes_goblins() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::mogg_war_marshal());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    let goblins = |g: &GameState| g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.definition.name == "Goblin").count();
    assert_eq!(goblins(&g), 1, "ETB Goblin");
    g.remove_to_graveyard_with_triggers(id);
    drain_stack(&mut g);
    assert_eq!(goblins(&g), 2, "death Goblin");
}

/// Seasonal Ritual (Rosethorn Acolyte) adds one mana of any color.
#[test]
fn adventure_rosethorn_acolyte_seasonal_ritual_ramps() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::rosethorn_acolyte());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::CastAdventure {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Seasonal Ritual");
    drain_stack(&mut g);
    assert_eq!(g.players[0].mana_pool.total(), 1, "ritual added one mana");
    assert!(g.exile.iter().any(|c| c.id == id && c.on_adventure), "exiled on adventure");
}

/// Vampire of the Dire Moon is a {B} 1/1 with deathtouch + lifelink.
#[test]
fn vampire_of_the_dire_moon_keywords() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::vampire_of_the_dire_moon());
    let c = g.battlefield_find(id).unwrap();
    assert_eq!((c.power(), c.toughness()), (1, 1));
    assert!(c.definition.keywords.contains(&Keyword::Deathtouch));
    assert!(c.definition.keywords.contains(&Keyword::Lifelink));
}

/// Accorder Paladin's Battle Cry pumps other attackers by +1/+0.
#[test]
fn accorder_paladin_battle_cry_pumps_team() {
    let mut g = two_player_game();
    let paladin = g.add_card_to_battlefield(0, catalog::accorder_paladin());
    let ally = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(paladin);
    g.clear_sickness(ally);
    g.active_player_idx = 0;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![
        Attack { attacker: paladin, target: AttackTarget::Player(1) },
        Attack { attacker: ally, target: AttackTarget::Player(1) },
    ])).unwrap();
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(ally).unwrap().power, 3, "other attacker got +1/+0");
}

/// Precinct Captain makes a Soldier when it connects.
#[test]
fn precinct_captain_makes_soldier_on_combat_damage() {
    let mut g = two_player_game();
    let cap = g.add_card_to_battlefield(0, catalog::precinct_captain());
    g.clear_sickness(cap);
    g.active_player_idx = 0;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![
        Attack { attacker: cap, target: AttackTarget::Player(1) },
    ])).unwrap();
    // Precinct Captain has first strike, so its damage lands in the
    // first-strike combat-damage step.
    // Precinct Captain has first strike — its damage lands in the
    // first-strike combat-damage step.
    g.step = TurnStep::FirstStrikeDamage;
    g.resolve_first_strike_damage().expect("first-strike damage");
    drain_stack(&mut g);
    assert_eq!(g.battlefield.iter().filter(|c| c.controller == 0
        && c.definition.name == "Soldier").count(), 1, "made a Soldier on hit");
}

/// Nivix Cyclops gets +3/+0 when you cast an instant or sorcery.
#[test]
fn nivix_cyclops_pumps_on_instant() {
    let mut g = two_player_game();
    let cy = g.add_card_to_battlefield(0, catalog::nivix_cyclops());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    cast_at(&mut g, bolt, Target::Player(1));
    assert_eq!(g.battlefield_find(cy).unwrap().power(), 4, "+3/+0 from the spell");
}

/// Festival Crasher grows a +1/+1 counter per instant/sorcery cast.
#[test]
fn festival_crasher_grows_on_spells() {
    let mut g = two_player_game();
    let fc = g.add_card_to_battlefield(0, catalog::festival_crasher());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    for _c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] { g.players[0].mana_pool.add(_c, 20); }
    g.players[0].mana_pool.add_colorless(20);
    cast_at(&mut g, bolt, Target::Player(1));
    let c = g.battlefield_find(fc).unwrap();
    assert_eq!((c.power(), c.toughness()), (2, 4), "permanent +1/+1 counter");
}

// ── CR 509.1b block-restriction keywords ────────────────────────────────────

/// Silhana Ledgewalker can't be blocked except by creatures with flying.
#[test]
fn silhana_ledgewalker_only_blocked_by_flyers() {
    let mut g = two_player_game();
    let ledge = g.add_card_to_battlefield(0, catalog::silhana_ledgewalker());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let angel = g.add_card_to_battlefield(1, catalog::serra_angel()); // flyer
    assert!(!g.blocker_can_block_attacker(bear, ledge), "ground can't block");
    assert!(g.blocker_can_block_attacker(angel, ledge), "flyer can block");
}

/// Steel Leaf Champion can't be blocked by creatures with power 2 or less.
#[test]
fn steel_leaf_champion_blocked_only_by_power_three_plus() {
    let mut g = two_player_game();
    let champ = g.add_card_to_battlefield(0, catalog::steel_leaf_champion());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let angel = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4
    assert!(!g.blocker_can_block_attacker(bear, champ), "power 2 can't block");
    assert!(g.blocker_can_block_attacker(angel, champ), "power 4 can block");
}

/// Thalia, Heretic Cathar taps opponents' creatures and nonbasic lands as
/// they enter; the controller's own and basic lands are unaffected.
#[test]
fn thalia_heretic_cathar_taps_opponent_creatures_and_nonbasics() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::thalia_heretic_cathar());
    // Opponent's creature enters tapped.
    let opp_bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.fire_self_etb_triggers(opp_bear, 1);
    assert!(g.battlefield_find(opp_bear).unwrap().tapped, "opp creature tapped");
    // Controller's own creature is unaffected.
    let own_bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.fire_self_etb_triggers(own_bear, 0);
    assert!(!g.battlefield_find(own_bear).unwrap().tapped, "own creature untapped");
    // Opponent's nonbasic land enters tapped; a basic does not.
    let nonbasic = g.add_card_to_battlefield(1, catalog::cephalid_coliseum());
    g.fire_self_etb_triggers(nonbasic, 1);
    assert!(g.battlefield_find(nonbasic).unwrap().tapped, "opp nonbasic land tapped");
    let basic = g.add_card_to_battlefield(1, catalog::island());
    g.fire_self_etb_triggers(basic, 1);
    assert!(!g.battlefield_find(basic).unwrap().tapped, "opp basic land untapped");
}

// ── Coldsteel Heart (choose-a-color mana rock) ──────────────────────────────

#[test]
fn coldsteel_heart_enters_tapped_and_taps_for_chosen_color() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Color(Color::Blue)]));
    let id = g.add_card_to_hand(0, catalog::coldsteel_heart());
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    let heart = g.battlefield_find(id).expect("resolved onto battlefield");
    assert_eq!(heart.chosen_color, Some(Color::Blue), "stamped the chosen color");
    assert!(heart.tapped, "Coldsteel Heart enters tapped");
    g.battlefield_find_mut(id).unwrap().tapped = false;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("tap for the chosen color");
    assert_eq!(g.players[0].mana_pool.amount(Color::Blue), 1);
}

#[test]
fn floodfarm_verge_blue_gated_on_plains_or_island() {
    let mut g = two_player_game();
    let v = g.add_card_to_battlefield(0, catalog::floodfarm_verge());
    // White is unconditional.
    g.perform_action(GameAction::ActivateAbility { card_id: v, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None })
        .expect("white");
    drain_stack(&mut g);
    assert!(g.players[0].mana_pool.amount(Color::White) > 0);
    // Blue needs a Plains or Island.
    g.battlefield_find_mut(v).unwrap().tapped = false;
    assert!(g
        .perform_action(GameAction::ActivateAbility { card_id: v, ability_index: 1, target: None, additional_targets: Vec::new(), x_value: None })
        .is_err());
    g.add_card_to_battlefield(0, catalog::island());
    g.perform_action(GameAction::ActivateAbility { card_id: v, ability_index: 1, target: None, additional_targets: Vec::new(), x_value: None })
        .expect("blue now allowed");
    drain_stack(&mut g);
    assert!(g.players[0].mana_pool.amount(Color::Blue) > 0);
}


#[test]
fn new_talismans_tap_for_colorless_and_their_two_colors() {
    type TwoColorCase = (fn() -> crabomination::card::CardDefinition, Color, Color);
    let cases: &[TwoColorCase] = &[
        (catalog::talisman_of_hierarchy, Color::White, Color::Black),
        (catalog::talisman_of_indulgence, Color::Black, Color::Red),
        (catalog::talisman_of_resilience, Color::Black, Color::Green),
        (catalog::talisman_of_impulse, Color::Red, Color::Green),
        (catalog::talisman_of_unity, Color::Green, Color::White),
    ];
    for (factory, c1, c2) in cases {
        let mut g = two_player_game();
        let id = g.add_card_to_battlefield(0, factory());
        g.clear_sickness(id);
        // Colorless ability (index 0).
        g.perform_action(GameAction::ActivateAbility {
            card_id: id, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None }).expect("colorless");
        assert_eq!(g.players[0].mana_pool.colorless_amount(), 1);
        // First color (index 1, costs 1 life).
        g.battlefield_find_mut(id).unwrap().tapped = false;
        let life = g.players[0].life;
        g.perform_action(GameAction::ActivateAbility {
            card_id: id, ability_index: 1, target: None, additional_targets: Vec::new(), x_value: None }).expect("c1");
        drain_stack(&mut g);
        assert_eq!(g.players[0].mana_pool.amount(*c1), 1);
        assert_eq!(g.players[0].life, life - 1);
        // Second color (index 2).
        g.battlefield_find_mut(id).unwrap().tapped = false;
        g.perform_action(GameAction::ActivateAbility {
            card_id: id, ability_index: 2, target: None, additional_targets: Vec::new(), x_value: None }).expect("c2");
        drain_stack(&mut g);
        assert_eq!(g.players[0].mana_pool.amount(*c2), 1);
    }
}

#[test]
fn signets_pay_one_and_tap_for_their_two_colors() {
    type TwoColorCase = (fn() -> crabomination::card::CardDefinition, Color, Color);
    let cases: &[TwoColorCase] = &[
        (catalog::azorius_signet, Color::White, Color::Blue),
        (catalog::dimir_signet, Color::Blue, Color::Black),
        (catalog::rakdos_signet, Color::Black, Color::Red),
        (catalog::gruul_signet, Color::Red, Color::Green),
        (catalog::selesnya_signet, Color::Green, Color::White),
        (catalog::orzhov_signet, Color::White, Color::Black),
        (catalog::izzet_signet, Color::Blue, Color::Red),
        (catalog::golgari_signet, Color::Black, Color::Green),
        (catalog::boros_signet, Color::Red, Color::White),
        (catalog::simic_signet, Color::Green, Color::Blue),
    ];
    for (factory, c1, c2) in cases {
        let mut g = two_player_game();
        let id = g.add_card_to_battlefield(0, factory());
        g.clear_sickness(id);
        g.players[0].mana_pool.add_colorless(1);
        g.perform_action(GameAction::ActivateAbility {
            card_id: id, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None }).expect("signet activates");
        drain_stack(&mut g);
        assert_eq!(g.players[0].mana_pool.amount(*c1), 1, "produced first color");
        assert_eq!(g.players[0].mana_pool.amount(*c2), 1, "produced second color");
        assert_eq!(g.players[0].mana_pool.colorless_amount(), 0, "spent the {{1}}");
    }
}

