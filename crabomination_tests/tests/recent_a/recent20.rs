//! Functionality tests for the `catalog::sets::decks::recent20` batch (OTJ:
//! commit a crime, pack tactics, outlaws).

use crabomination::card::Keyword;
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::types::{Attack, AttackTarget};
use crabomination::game::*;
use crabomination::mana::Color;
use crabomination::TurnStep;

/// Player 0 commits a crime by casting Lava Spike at player 1.
fn commit_crime(g: &mut GameState) {
    let ls = g.add_card_to_hand(0, catalog::lava_spike());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    cast_at(g, ls, Target::Player(1));
}

fn count_named(g: &GameState, controller: usize, name: &str) -> usize {
    g.battlefield.iter().filter(|c| c.controller == controller && c.definition.name == name).count()
}

fn add_top(g: &mut GameState, player: usize, def: crabomination::card::CardDefinition) {
    let id = g.next_id();
    g.players[player].add_to_library_top(id, def);
}

// ── Pack tactics ─────────────────────────────────────────────────────────────

/// Battle Cry Goblin makes a Goblin when you attack with total power 6+.
#[test]
fn battle_cry_goblin_pack_tactics_makes_token() {
    let mut g = two_player_game();
    let bcg = g.add_card_to_battlefield(0, catalog::battle_cry_goblin()); // 2
    let serra = g.add_card_to_battlefield(0, catalog::serra_angel()); // 4 → total 6
    g.clear_sickness(bcg);
    g.clear_sickness(serra);
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    while g.step != TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).unwrap();
    }
    g.perform_action(GameAction::DeclareAttackers(vec![
        Attack { attacker: bcg, target: AttackTarget::Player(1) },
        Attack { attacker: serra, target: AttackTarget::Player(1) },
    ]))
    .expect("attack");
    drain_stack(&mut g);
    assert_eq!(count_named(&g, 0, "Goblin"), 1, "pack tactics minted a Goblin");
}

/// Below the total-power threshold, Battle Cry Goblin makes nothing.
#[test]
fn battle_cry_goblin_no_token_below_threshold() {
    let mut g = two_player_game();
    let bcg = g.add_card_to_battlefield(0, catalog::battle_cry_goblin()); // 2 only
    g.clear_sickness(bcg);
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    while g.step != TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).unwrap();
    }
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: bcg,
        target: AttackTarget::Player(1),
    }]))
    .expect("attack");
    drain_stack(&mut g);
    assert_eq!(count_named(&g, 0, "Goblin"), 0, "total power 2 < 6, no token");
}

// ── Commit a crime ───────────────────────────────────────────────────────────

/// Gisa makes two Zombie Rogues when you commit a crime.
#[test]
fn gisa_crime_makes_two_zombies() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::gisa_the_hellraiser());
    commit_crime(&mut g);
    assert_eq!(count_named(&g, 0, "Zombie Rogue"), 2, "two tokens from the crime");
}

/// Gisa's crime trigger fires only once each turn.
#[test]
fn gisa_crime_once_per_turn() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::gisa_the_hellraiser());
    commit_crime(&mut g);
    commit_crime(&mut g);
    assert_eq!(count_named(&g, 0, "Zombie Rogue"), 2, "second crime same turn does nothing");
}

/// Gisa buffs your Zombies and grants them menace.
#[test]
fn gisa_anthem_buffs_undead() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::gisa_the_hellraiser());
    let zombie = g.add_card_to_battlefield(0, catalog::gravecrawler());
    let cp = g.computed_permanent(zombie).unwrap();
    assert!(cp.power >= 3, "Gravecrawler 2/1 → 3/2 under Gisa");
    assert!(cp.keywords.contains(&Keyword::Menace), "gains menace");
}

/// Magda makes a tapped Treasure when you commit a crime.
#[test]
fn magda_crime_makes_treasure() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::magda_the_hoardmaster());
    commit_crime(&mut g);
    let t = g.battlefield.iter().find(|c| c.controller == 0 && c.definition.name == "Treasure");
    assert!(t.is_some(), "a Treasure was made");
    assert!(t.unwrap().tapped, "the Treasure is tapped");
}

/// Magda sacrifices three Treasures to make a 4/4 flying, hasty Scorpion Dragon.
#[test]
fn magda_sacs_three_treasures_for_scorpion_dragon() {
    let mut g = two_player_game();
    let magda = g.add_card_to_battlefield(0, catalog::magda_the_hoardmaster());
    g.clear_sickness(magda);
    let treasure = crabomination::game::effects::treasure_token();
    for _ in 0..3 {
        g.add_token_to_battlefield(0, &treasure);
    }
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: magda,
        ability_index: 0,
        target: None,
        additional_targets: Vec::new(),
        x_value: None,
    })
    .expect("sac three Treasures");
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield.iter().filter(|c| c.controller == 0 && c.definition.name == "Treasure").count(),
        0,
        "all three Treasures sacrificed"
    );
    let dragon = g
        .battlefield
        .iter()
        .find(|c| c.definition.name == "Scorpion Dragon")
        .expect("4/4 Scorpion Dragon minted");
    assert_eq!((dragon.definition.power, dragon.definition.toughness), (4, 4));
    assert!(dragon.definition.keywords.contains(&Keyword::Flying));
    assert!(dragon.definition.keywords.contains(&Keyword::Haste));
}

/// Marchesa digs two when you commit a crime and pay {1}.
#[test]
fn marchesa_crime_digs() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::marchesa_dealer_of_death());
    // stock the library so the dig has cards.
    add_top(&mut g, 0, catalog::grizzly_bears());
    add_top(&mut g, 0, catalog::grizzly_bears());
    let hand_before = g.players[0].hand.len();
    let gy_before = g.players[0].graveyard.len();
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    g.players[0].mana_pool.add_colorless(1);
    commit_crime(&mut g);
    assert_eq!(g.players[0].hand.len(), hand_before + 1, "one card to hand");
    assert!(g.players[0].graveyard.len() > gy_before, "one card to graveyard");
}

/// Forsaken Miner returns from the graveyard when you commit a crime and pay {B}.
#[test]
fn forsaken_miner_returns_on_crime() {
    let mut g = two_player_game();
    let miner = g.add_card_to_graveyard(0, catalog::forsaken_miner());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    g.players[0].mana_pool.add(Color::Black, 1);
    commit_crime(&mut g);
    assert!(g.battlefield_find(miner).is_some(), "Forsaken Miner returned to the battlefield");
}

/// Nimble Brigand is unblockable after you've committed a crime.
#[test]
fn nimble_brigand_unblockable_after_crime() {
    let mut g = two_player_game();
    let nb = g.add_card_to_battlefield(0, catalog::nimble_brigand());
    assert!(!g.computed_permanent(nb).unwrap().keywords.contains(&Keyword::Unblockable));
    commit_crime(&mut g);
    assert!(
        g.computed_permanent(nb).unwrap().keywords.contains(&Keyword::Unblockable),
        "unblockable once a crime is committed"
    );
}

// ── Outlaw matters ───────────────────────────────────────────────────────────

/// Vial Smasher pings an opponent when another outlaw you control enters.
#[test]
fn vial_smasher_pings_on_outlaw_etb() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::vial_smasher_gleeful_grenadier());
    let life_before = g.players[1].life;
    // Treasure Dredger is a Rogue (outlaw).
    let dredger = g.add_card_to_battlefield(0, catalog::treasure_dredger());
    g.dispatch_triggers_for_events(&[GameEvent::PermanentEntered { card_id: dredger }]);
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life_before - 1, "1 damage to the opponent");
}

/// Rakish Crew drains when an outlaw you control dies.
#[test]
fn rakish_crew_drains_on_outlaw_death() {
    let mut g = two_player_game();
    let crew = g.add_card_to_battlefield(0, catalog::rakish_crew());
    g.fire_self_etb_triggers(crew, 0); // ETB Mercenary token
    drain_stack(&mut g);
    let rogue = g.add_card_to_battlefield(0, catalog::treasure_dredger()); // Rogue
    let opp_life = g.players[1].life;
    let mut evs = g.remove_to_graveyard_with_triggers(rogue);
    evs.push(GameEvent::CreatureDied { card_id: rogue });
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_life - 1, "opponent loses 1");
}

// ── Plot / value ─────────────────────────────────────────────────────────────

/// Rictus Robber makes a Zombie when a creature died this turn.
#[test]
fn rictus_robber_token_if_creature_died() {
    let mut g = two_player_game();
    let chump = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.remove_to_graveyard_with_triggers(chump);
    drain_stack(&mut g);
    let robber = g.add_card_to_battlefield(0, catalog::rictus_robber());
    g.fire_self_etb_triggers(robber, 0);
    drain_stack(&mut g);
    assert_eq!(count_named(&g, 0, "Zombie Rogue"), 1, "a creature died → token");
}

// ── Simple staples ───────────────────────────────────────────────────────────

/// Holy Cow gains 2 life on ETB.
#[test]
fn holy_cow_gains_life() {
    let mut g = two_player_game();
    let life = g.players[0].life;
    let cow = g.add_card_to_battlefield(0, catalog::holy_cow());
    g.fire_self_etb_triggers(cow, 0);
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life + 2, "gained 2 life");
}

/// Sterling Keykeeper taps a target creature.
#[test]
fn sterling_keykeeper_taps_creature() {
    let mut g = two_player_game();
    let keeper = g.add_card_to_battlefield(0, catalog::sterling_keykeeper());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(keeper);
    g.players[0].mana_pool.add_colorless(2);
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: keeper,
        ability_index: 0,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        x_value: None,
    })
    .expect("activate");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).unwrap().tapped, "bear tapped");
}

/// Treasure Dredger mints a Treasure.
#[test]
fn treasure_dredger_makes_treasure() {
    let mut g = two_player_game();
    let td = g.add_card_to_battlefield(0, catalog::treasure_dredger());
    g.clear_sickness(td);
    g.players[0].mana_pool.add_colorless(1);
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: td,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
    })
    .expect("activate");
    drain_stack(&mut g);
    assert_eq!(count_named(&g, 0, "Treasure"), 1);
}

/// Slick Sequence draws if you've cast another spell this turn.
#[test]
fn slick_sequence_draws_after_second_spell() {
    let mut g = two_player_game();
    g.players[0].spells_cast_this_turn = 1; // pretend a spell was already cast
    add_top(&mut g, 0, catalog::grizzly_bears());
    let hand_before = g.players[0].hand.len();
    let ss = g.add_card_to_hand(0, catalog::slick_sequence());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    cast_at(&mut g, ss, Target::Player(1));
    // cast itself bumps the count to 2; the "another spell" gate (>=1 prior) holds.
    assert_eq!(g.players[0].hand.len(), hand_before + 1, "drew a card");
}

/// Razzle-Dazzler grows and turns unblockable on your second spell.
#[test]
fn razzle_dazzler_grows_on_second_spell() {
    let mut g = two_player_game();
    let rd = g.add_card_to_battlefield(0, catalog::razzle_dazzler());
    g.players[0].spells_cast_this_turn = 1;
    // Casting any spell makes it the second this turn.
    let bolt = g.add_card_to_hand(0, catalog::lava_spike());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    cast_at(&mut g, bolt, Target::Player(1));
    let cp = g.computed_permanent(rd).unwrap();
    assert_eq!(cp.power, 2, "got a +1/+1 counter");
    assert!(cp.keywords.contains(&Keyword::Unblockable), "can't be blocked this turn");
}

/// Quilled Charger pumps and gains menace when it attacks while saddled.
#[test]
fn quilled_charger_pumps_while_saddled() {
    let mut g = two_player_game();
    let qc = g.add_card_to_battlefield(0, catalog::quilled_charger());
    g.battlefield_find_mut(qc).unwrap().saddled = true;
    g.clear_sickness(qc);
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    while g.step != TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).unwrap();
    }
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: qc,
        target: AttackTarget::Player(1),
    }]))
    .expect("attack");
    drain_stack(&mut g);
    let cp = g.computed_permanent(qc).unwrap();
    assert_eq!((cp.power, cp.toughness), (5, 5), "+1/+2 while saddled");
    assert!(cp.keywords.contains(&Keyword::Menace));
}

/// Lassoed by the Law exiles an opponent's permanent and makes a Mercenary.
#[test]
fn lassoed_exiles_and_makes_token() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let lasso = g.add_card_to_battlefield(0, catalog::lassoed_by_the_law());
    g.fire_self_etb_triggers(lasso, 0);
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none(), "opponent's bear exiled");
    assert_eq!(count_named(&g, 0, "Mercenary"), 1, "made a Mercenary");
}

/// Roxanne makes a Meteorite when she enters.
#[test]
fn roxanne_makes_meteorite_on_etb() {
    let mut g = two_player_game();
    let roxanne = g.add_card_to_battlefield(0, catalog::roxanne_starfall_savant());
    g.fire_self_etb_triggers(roxanne, 0);
    drain_stack(&mut g);
    assert_eq!(count_named(&g, 0, "Meteorite"), 1);
}

/// Honest Rutstein returns a creature card from your graveyard and cheapens
/// creature spells.
#[test]
fn honest_rutstein_value() {
    use crabomination::game::actions::cost_reduction_for_spell;
    let mut g = two_player_game();
    g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let rutstein = g.add_card_to_battlefield(0, catalog::honest_rutstein());
    g.fire_self_etb_triggers(rutstein, 0);
    drain_stack(&mut g);
    assert!(
        g.players[0].hand.iter().any(|c| c.definition.name == "Grizzly Bears"),
        "returned the creature card to hand"
    );
    let bears = crabomination::card::CardInstance::new(g.next_id(), catalog::grizzly_bears(), 0);
    assert_eq!(cost_reduction_for_spell(&g, 0, &bears, None), 1, "creature spells cost {{1}} less");
}

/// Stoic Sphinx has hexproof until you cast a spell.
#[test]
fn stoic_sphinx_hexproof_until_spell() {
    let mut g = two_player_game();
    let sphinx = g.add_card_to_battlefield(0, catalog::stoic_sphinx());
    assert!(g.computed_permanent(sphinx).unwrap().keywords.contains(&Keyword::Hexproof));
    g.players[0].spells_cast_this_turn = 1;
    assert!(
        !g.computed_permanent(sphinx).unwrap().keywords.contains(&Keyword::Hexproof),
        "loses hexproof once you've cast a spell"
    );
}

/// Hellspur Brute gets Affinity for outlaws.
#[test]
fn hellspur_brute_affinity_for_outlaws() {
    use crabomination::game::actions::cost_reduction_for_spell;
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::treasure_dredger()); // Rogue
    g.add_card_to_battlefield(0, catalog::nimble_brigand()); // Rogue
    let brute = crabomination::card::CardInstance::new(g.next_id(), catalog::hellspur_brute(), 0);
    assert_eq!(cost_reduction_for_spell(&g, 0, &brute, None), 2, "{{1}} less per outlaw");
}

/// Bovine Intervention destroys and gives the controller an Ox.
#[test]
fn bovine_intervention_destroys_and_makes_ox() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let bovine = g.add_card_to_hand(0, catalog::bovine_intervention());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    cast_at(&mut g, bovine, Target::Permanent(bear));
    assert!(g.battlefield_find(bear).is_none(), "bear destroyed");
    assert_eq!(count_named(&g, 1, "Ox"), 1, "its controller made an Ox");
}
