//! Functionality tests for the `catalog::sets::decks::recent3` batch.

use crate::card::{CounterType, Keyword};
use crate::catalog;
use crate::game::types::{Attack, AttackTarget, TurnStep};
use crate::game::*;
use crate::mana::Color;

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
    use crate::card::{ActivatedAbility, CardDefinition, CardType, Effect, Selector, Value};
    use crate::effect::PlayerRef;
    use crate::server::bot::{Bot, RandomBot};
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
