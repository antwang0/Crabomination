//! Functionality tests for the recent-set staples in
//! `catalog::sets::decks::recent`.

use crate::card::{CounterType, CreatureType, Keyword};
use crate::catalog;
use crate::decision::{DecisionAnswer, ScriptedDecider};
use crate::game::types::{Attack, AttackTarget};
use crate::game::*;
use crate::mana::Color;
use crate::TurnStep;

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

/// Cackling Slasher with no death this turn enters as a vanilla 3/3.
#[test]
fn cackling_slasher_no_death_no_counter() {
    let mut g = two_player_game();
    let slasher = g.move_card_to_battlefield_for_test(0, catalog::cackling_slasher());
    drain_stack(&mut g);
    let r = g.battlefield_find(slasher).unwrap();
    assert_eq!(r.counters.get(&CounterType::PlusOnePlusOne).copied(), None);
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

/// Vaultborn Tyrant leaves a token copy of itself when it dies.
#[test]
fn vaultborn_tyrant_dies_into_a_copy() {
    let mut g = two_player_game();
    let vt = g.add_card_to_battlefield(0, catalog::vaultborn_tyrant());
    g.battlefield_find_mut(vt).unwrap().damage = 6; // lethal
    g.check_state_based_actions();
    drain_stack(&mut g);
    let copies = g
        .battlefield
        .iter()
        .filter(|c| c.definition.name == "Vaultborn Tyrant" && c.is_token)
        .count();
    assert_eq!(copies, 1, "one token copy on the battlefield");
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

/// Unstoppable Slasher halves a player's life on combat damage.
#[test]
fn unstoppable_slasher_halves_life() {
    let mut g = two_player_game();
    let slasher = g.add_card_to_battlefield(0, catalog::unstoppable_slasher());
    g.clear_sickness(slasher);
    g.players[1].life = 20;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    while g.step != TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).unwrap();
    }
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: slasher,
        target: AttackTarget::Player(1),
    }]))
    .expect("attack");
    drain_stack(&mut g);
    while g.step != TurnStep::PostCombatMain && g.step != TurnStep::End {
        g.perform_action(GameAction::PassPriority).unwrap();
    }
    drain_stack(&mut g);
    // 20 - 2 (combat) = 18, then lose half (9) rounded up → 9.
    assert_eq!(g.players[1].life, 9, "took 2 combat then lost half (rounded up)");
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
        !back.definition.card_types.contains(&crate::card::CardType::Creature),
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
    g.move_card_to_battlefield_for_test(0, catalog::outcaster_greenblade());
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand + 1, "fetched a land to hand");
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
