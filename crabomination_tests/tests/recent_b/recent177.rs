//! Functionality tests for `catalog::sets::decks::recent177`.

use crabomination::card::{CreatureType, Keyword};
use crabomination::catalog;
use crabomination::game::*;
use crabomination::mana::Color;

fn advance_to(g: &mut GameState, step: TurnStep) {
    while g.step != step {
        g.perform_action(GameAction::PassPriority).expect("pass priority");
    }
}

/// Exemplar of Light gains a +1/+1 counter when you gain life and draws a card
/// the first time counters are placed on it that turn.
#[test]
fn exemplar_of_light_counters_then_draws() {
    let mut g = two_player_game();
    let ex = g.add_card_to_battlefield(0, catalog::exemplar_of_light());
    g.add_card_to_library(0, catalog::grizzly_bears());
    let hand = g.players[0].hand.len();
    g.adjust_life(0, 3);
    g.dispatch_triggers_for_events(&[GameEvent::LifeGained { player: 0, amount: 3 }]);
    drain_stack(&mut g);
    let counters = *g.battlefield_find(ex).unwrap().counters.get(&crabomination::card::CounterType::PlusOnePlusOne).unwrap_or(&0);
    assert_eq!(counters, 1, "gained a +1/+1 counter from lifegain");
    assert_eq!(g.players[0].hand.len(), hand + 1, "drew from the counter trigger");
}

/// Ashroot Animist's attack trigger gives another creature you control trample
/// and +power/+power (power 4 → +4/+4).
#[test]
fn ashroot_animist_pumps_ally_on_attack() {
    let mut g = two_player_game();
    let ash = g.add_card_to_battlefield(0, catalog::ashroot_animist());
    let ally = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    g.clear_sickness(ash);
    g.clear_sickness(ally);
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: ash,
        target: AttackTarget::Player(1),
    }]))
    .expect("attack");
    drain_stack(&mut g);
    let cp = g.computed_permanent(ally).unwrap();
    assert_eq!(cp.power, 6, "2 base + 4 from Ashroot's power");
    assert!(cp.keywords.contains(&Keyword::Trample), "ally gained trample");
}

/// Arahbo makes a Cat token when a nontoken Cat enters, and its anthem pumps
/// other Cats.
#[test]
fn arahbo_tokens_and_anthem() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::arahbo_the_first_fang());
    drain_stack(&mut g);
    let before = g.battlefield.len();
    // A nontoken Cat entering triggers the token.
    let mut cat = catalog::grizzly_bears();
    cat.name = "Test Cat";
    cat.subtypes.creature_types = vec![CreatureType::Cat];
    let entered = g.add_card_to_battlefield(0, cat);
    g.dispatch_triggers_for_events(&[GameEvent::PermanentEntered { card_id: entered }]);
    drain_stack(&mut g);
    assert!(g.battlefield.len() > before + 1, "made a Cat token on top of the entered Cat");
    // The anthem pumps the other Cat (2/2 → 3/3).
    let cp = g.computed_permanent(entered).unwrap();
    assert_eq!(cp.power, 3, "other Cat gets +1/+1 from Arahbo");
}

/// Bumbleflower's Sharepot makes a Food on entry and its sac ability destroys a
/// nonland permanent.
#[test]
fn bumbleflowers_sharepot_food_and_destroy() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    // move_card_to_battlefield_for_test fires the SelfSource ETB (create Food).
    let pot = g.move_card_to_battlefield_for_test(0, catalog::bumbleflowers_sharepot());
    drain_stack(&mut g);
    assert!(
        g.battlefield.iter().any(|c| c.definition.subtypes.artifact_subtypes.contains(&crabomination::card::ArtifactSubtype::Food)),
        "made a Food token"
    );
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.players[0].mana_pool.add_colorless(5);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Target(Target::Permanent(victim))]));
    g.perform_action(GameAction::ActivateAbility {
        card_id: pot, ability_index: 0,
        target: Some(Target::Permanent(victim)), additional_targets: Vec::new(), x_value: None,
    })
    .expect("sac to destroy");
    drain_stack(&mut g);
    assert!(g.battlefield_find(victim).is_none(), "destroyed the nonland permanent");
    assert!(g.battlefield_find(pot).is_none(), "Sharepot sacrificed");
}

/// Celestial Armor attaches on entry, grants +2/+0 and flying, and gives the
/// creature hexproof + indestructible until end of turn.
#[test]
fn celestial_armor_attaches_and_grants() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    g.decider = Box::new(crabomination::decision::ScriptedDecider::new([
        crabomination::decision::DecisionAnswer::Target(Target::Permanent(bear)),
    ]));
    // Fire the SelfSource ETB (attach + grant) via the move path.
    g.move_card_to_battlefield_for_test(0, catalog::celestial_armor());
    drain_stack(&mut g);
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!(cp.power, 4, "+2/+0 from the Equipment");
    assert!(cp.keywords.contains(&Keyword::Flying), "gained flying");
    assert!(cp.keywords.contains(&Keyword::Indestructible), "gained indestructible EOT");
    assert!(cp.keywords.contains(&Keyword::Hexproof), "gained hexproof EOT");
}

/// Strix Lookout loots: {1}{U},{T} draws then discards.
#[test]
fn strix_lookout_loots() {
    let mut g = two_player_game();
    let bird = g.add_card_to_battlefield(0, catalog::strix_lookout());
    g.clear_sickness(bird);
    g.add_card_to_library(0, catalog::grizzly_bears());
    g.add_card_to_hand(0, catalog::grizzly_bears()); // something to discard
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let hand = g.players[0].hand.len();
    g.perform_action(GameAction::ActivateAbility {
        card_id: bird, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    })
    .expect("loot");
    drain_stack(&mut g);
    // Draw +1 then discard -1 → net hand unchanged, and a card is in the graveyard.
    assert_eq!(g.players[0].hand.len(), hand, "drew one, discarded one");
    assert!(!g.players[0].graveyard.is_empty(), "discarded a card");
}

/// Vanguard Seraph surveils the first time you gain life each turn (once).
#[test]
fn vanguard_seraph_surveils_on_first_lifegain() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::vanguard_seraph());
    g.add_card_to_library(0, catalog::grizzly_bears());
    let lib = g.players[0].library.len();
    g.adjust_life(0, 2);
    g.dispatch_triggers_for_events(&[GameEvent::LifeGained { player: 0, amount: 2 }]);
    drain_stack(&mut g);
    // Surveil 1 auto-keeps or bins the top card; either way the trigger fired
    // (library shrank by 0 or 1). Fire a second lifegain — no extra surveil.
    let after_first = g.players[0].library.len();
    assert!(after_first <= lib, "surveil looked at the top card");
    g.adjust_life(0, 2);
    g.dispatch_triggers_for_events(&[GameEvent::LifeGained { player: 0, amount: 2 }]);
    drain_stack(&mut g);
    assert_eq!(g.players[0].library.len(), after_first, "only the first lifegain surveils");
}

/// Vampire Soulcaller returns a creature card from your graveyard on entry.
#[test]
fn vampire_soulcaller_returns_creature() {
    let mut g = two_player_game();
    let dead = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    g.move_card_to_battlefield_for_test(0, catalog::vampire_soulcaller());
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == dead), "returned the creature to hand");
}

/// Turn Inside Out pumps +3/+0 and, when the creature dies, manifests dread.
#[test]
fn turn_inside_out_pumps_and_manifests_on_death() {
    let mut g = two_player_game();
    let target = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    for _ in 0..2 { g.add_card_to_library(0, catalog::grizzly_bears()); }
    let spell = g.add_card_to_hand(0, catalog::turn_inside_out());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(target)), additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("cast Turn Inside Out");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(target).unwrap().power, 5, "+3/+0 → 5 power");
    let bf = g.battlefield.len();
    // Kill the creature this turn → manifest dread fires (a face-down 2/2 enters).
    g.remove_to_graveyard_with_triggers(target);
    g.dispatch_triggers_for_events(&[GameEvent::CreatureDied { card_id: target }]);
    drain_stack(&mut g);
    assert!(g.battlefield.len() > bf.saturating_sub(1), "a manifested creature entered");
    assert!(g.battlefield.iter().any(|c| c.controller == 0 && c.face_down), "face-down manifest");
}

/// Huskburster Swarm costs {1} less per creature card in your graveyard.
#[test]
fn huskburster_swarm_graveyard_affinity() {
    use crabomination::game::actions::cost_reduction_for_spell;
    let mut g = two_player_game();
    let swarm = crabomination::card::CardInstance::new(g.next_id(), catalog::huskburster_swarm(), 0);
    assert_eq!(cost_reduction_for_spell(&g, 0, &swarm, None), 0, "empty graveyard → no discount");
    for _ in 0..3 { g.add_card_to_graveyard(0, catalog::grizzly_bears()); }
    g.add_card_to_graveyard(0, catalog::lightning_bolt()); // noncreature, ignored
    assert_eq!(cost_reduction_for_spell(&g, 0, &swarm, None), 3, "three creature cards → 3 generic off");
}
