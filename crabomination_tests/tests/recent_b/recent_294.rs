//! Tests for the recent294 Ravnica batch 4 (Simic Graft, Orzhov/Boros, utility).

use crabomination::card::Keyword;
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::types::{Attack, AttackTarget, Target};
use crabomination::game::{drain_stack, two_player_game, GameAction, GameState, TurnStep};
use crabomination::mana::Color;

fn flood(g: &mut GameState) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[0].mana_pool.add(c, 6);
    }
    g.players[0].mana_pool.add_colorless(8);
}

fn count_tokens(g: &GameState, name: &str) -> usize {
    g.battlefield.iter().filter(|c| c.is_token && c.definition.name == name).count()
}

#[test]
fn simic_ragworm_untaps_itself() {
    let mut g = two_player_game();
    let worm = g.add_card_to_battlefield(0, catalog::simic_ragworm());
    g.battlefield_find_mut(worm).unwrap().tapped = true;
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: worm, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("untap");
    drain_stack(&mut g);
    assert!(!g.battlefield_find(worm).unwrap().tapped, "Ragworm untapped itself");
}

#[test]
fn sporeback_troll_regenerates_a_counter_bearer() {
    let mut g = two_player_game();
    let troll = g.move_card_to_battlefield_for_test(0, catalog::sporeback_troll());
    assert_eq!(g.computed_permanent(troll).unwrap().power, 2, "Graft 2 → 2/2");
    g.clear_sickness(troll);
    // A creature with a +1/+1 counter is a legal regen target; one without isn't.
    let bare = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    flood(&mut g);
    assert!(g.perform_action(GameAction::ActivateAbility {
        card_id: troll, ability_index: 0, target: Some(Target::Permanent(bare)),
        additional_targets: vec![], x_value: None,
    }).is_err(), "a creature with no +1/+1 counter can't be targeted");
    g.perform_action(GameAction::ActivateAbility {
        card_id: troll, ability_index: 0, target: Some(Target::Permanent(troll)),
        additional_targets: vec![], x_value: None,
    }).expect("the graft creature carries counters — a legal target");
    drain_stack(&mut g);
    assert!(g.battlefield_find(troll).unwrap().regeneration_shields > 0, "regen shield up");
}

#[test]
fn silhana_starfletcher_taps_for_the_chosen_color() {
    let mut g = two_player_game();
    let fletcher = g.add_card_to_battlefield(0, catalog::silhana_starfletcher());
    // Choose blue as it enters.
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Color(Color::Blue)]));
    g.fire_self_etb_triggers(fletcher, 0);
    drain_stack(&mut g);
    g.clear_sickness(fletcher);
    g.perform_action(GameAction::ActivateAbility {
        card_id: fletcher, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("tap for mana");
    assert_eq!(g.players[0].mana_pool.amount(Color::Blue), 1, "added the chosen blue mana");
}

#[test]
fn plaxmanta_shrouds_your_team_and_needs_green() {
    // No green spent → Plaxmanta sacrifices itself; the shroud grant still lands.
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let plax = g.add_card_to_hand(0, catalog::plaxmanta());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: plax, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    assert!(g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Shroud),
        "your creatures gained shroud");
    assert!(g.battlefield_find(plax).is_none(), "sacrificed — no green was spent");

    // Green spent → it sticks around.
    let mut g = two_player_game();
    let plax = g.add_card_to_hand(0, catalog::plaxmanta());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: plax, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(plax).is_some(), "green spent → it stays");
}

#[test]
fn skeletal_vampire_broods_bats_and_regenerates() {
    let mut g = two_player_game();
    let vamp = g.add_card_to_battlefield(0, catalog::skeletal_vampire());
    g.fire_self_etb_triggers(vamp, 0);
    drain_stack(&mut g);
    assert_eq!(count_tokens(&g, "Bat"), 2, "ETB made two Bats");
    g.clear_sickness(vamp);
    flood(&mut g);
    // {3}{B}{B}, Sacrifice a Bat: make two more Bats (net +1 Bat).
    g.perform_action(GameAction::ActivateAbility {
        card_id: vamp, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("bat-for-bats");
    drain_stack(&mut g);
    assert_eq!(count_tokens(&g, "Bat"), 3, "sacked one Bat, minted two");
    // Sacrifice a Bat: regenerate.
    g.perform_action(GameAction::ActivateAbility {
        card_id: vamp, ability_index: 1, target: None, additional_targets: vec![], x_value: None,
    }).expect("regen");
    drain_stack(&mut g);
    assert!(g.battlefield_find(vamp).unwrap().regeneration_shields > 0, "regen shield up");
    assert_eq!(count_tokens(&g, "Bat"), 2, "regen sacked another Bat");
}

#[test]
fn divebomber_griffin_snipes_an_attacker() {
    let mut g = two_player_game();
    g.active_player_idx = 1;
    let attacker = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(attacker);
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker, target: AttackTarget::Player(0),
    }])).expect("attack");
    let griffin = g.add_card_to_battlefield(0, catalog::divebomber_griffin());
    g.clear_sickness(griffin);
    g.priority.player_with_priority = 0; // the defender responds
    g.perform_action(GameAction::ActivateAbility {
        card_id: griffin, ability_index: 0, target: Some(Target::Permanent(attacker)),
        additional_targets: vec![], x_value: None,
    }).expect("dive");
    drain_stack(&mut g);
    assert!(g.battlefield_find(attacker).is_none(), "3 damage killed the 2/2 attacker");
    assert!(g.battlefield_find(griffin).is_none(), "Griffin sacrificed itself");
}

#[test]
fn scorched_rusalka_pings_a_player() {
    let mut g = two_player_game();
    let rusalka = g.add_card_to_battlefield(0, catalog::scorched_rusalka());
    let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(rusalka);
    g.players[0].mana_pool.add(Color::Red, 1);
    let life = g.players[1].life;
    g.perform_action(GameAction::ActivateAbility {
        card_id: rusalka, ability_index: 0, target: Some(Target::Player(1)),
        additional_targets: vec![], x_value: None,
    }).expect("ping");
    drain_stack(&mut g);
    assert!(g.battlefield_find(fodder).is_none(), "a creature was sacrificed");
    assert_eq!(g.players[1].life, life - 1, "1 damage to the target player");
}

#[test]
fn withstand_prevents_damage_and_draws() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::withstand());
    flood(&mut g);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(bear)), additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand_before, "drew one, spent one (net same)");
    // The bear now soaks the next 3 damage.
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(bear)), additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("bolt the shielded bear");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_some(), "3 prevented → the bear survives Lightning Bolt");
}

#[test]
fn steeple_roc_and_snapping_drake_fly() {
    let mut g = two_player_game();
    let roc = g.add_card_to_battlefield(0, catalog::steeple_roc());
    let drake = g.add_card_to_battlefield(0, catalog::snapping_drake());
    let rc = g.computed_permanent(roc).unwrap();
    assert!(rc.keywords.contains(&Keyword::Flying) && rc.keywords.contains(&Keyword::FirstStrike));
    assert_eq!((rc.power, rc.toughness), (3, 1));
    let dc = g.computed_permanent(drake).unwrap();
    assert!(dc.keywords.contains(&Keyword::Flying));
    assert_eq!((dc.power, dc.toughness), (3, 2));
}
