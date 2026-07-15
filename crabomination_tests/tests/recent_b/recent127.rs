//! Functionality tests for `catalog::sets::decks::recent127`.

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

/// Colossal Rattlewurm can be cast at instant speed only while you control a
/// Desert.
#[test]
fn colossal_rattlewurm_conditional_flash() {
    // No Desert → sorcery speed only; casting on the opponent's turn is rejected.
    let mut g = two_player_game();
    g.active_player_idx = 1;
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    let worm = g.add_card_to_hand(0, catalog::colossal_rattlewurm());
    g.players[0].mana_pool.add(Color::Green, 4);
    assert!(
        g.perform_action(GameAction::CastSpell {
            card_id: worm,
            target: None,
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .is_err(),
        "no Desert → can't cast at instant speed",
    );
    // With a Desert in play → flash lets it cast on the opponent's turn.
    g.add_card_to_battlefield(0, catalog::bristling_backwoods());
    g.players[0].mana_pool.add(Color::Green, 4);
    g.perform_action(GameAction::CastSpell {
        card_id: worm,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Desert in play → flash");
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
