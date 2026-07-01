//! Functionality tests for `catalog::sets::decks::recent59` — spellslinger/tempo.

use crate::card::{CounterType, Keyword};
use crate::catalog;
use crate::decision::{DecisionAnswer, ScriptedDecider};
use crate::game::*;

fn cast_sorcery(g: &mut GameState, controller: usize, id: CardId, x: Option<u32>) {
    g.step = TurnStep::PreCombatMain;
    g.active_player_idx = controller;
    g.priority.player_with_priority = controller;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: x,
    }).expect("cast");
    drain_stack(g);
}

#[test]
fn sky_terror_has_flying_and_menace() {
    let mut g = two_player_game();
    let st = g.add_card_to_battlefield(0, catalog::sky_terror());
    let cp = g.compute_battlefield();
    let c = cp.iter().find(|c| c.id == st).unwrap();
    assert!(c.keywords.contains(&Keyword::Flying) && c.keywords.contains(&Keyword::Menace));
}

#[test]
fn talrands_invocation_makes_two_drakes() {
    let mut g = two_player_game();
    let inv = g.add_card_to_hand(0, catalog::talrands_invocation());
    g.players[0].mana_pool.add(crate::mana::Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(2);
    cast_sorcery(&mut g, 0, inv, None);
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Drake").count(), 2);
}

#[test]
fn ondu_cleric_gains_life_per_ally() {
    use crate::card::{CardType, CreatureType, Subtypes};
    let ally = || crate::card::CardDefinition {
        name: "Ally Buddy",
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Ally], ..Default::default() },
        power: 1, toughness: 1, ..Default::default()
    };
    let mut g = two_player_game();
    // Two vanilla Allies out; a single Ondu Cleric (also an Ally) enters →
    // 3 Allies total → gain 3. Only the Ondu carries the trigger.
    g.add_card_to_battlefield(0, ally());
    g.add_card_to_battlefield(0, ally());
    let life = g.players[0].life;
    let c = g.add_card_to_battlefield(0, catalog::ondu_cleric());
    g.dispatch_triggers_for_events(&[GameEvent::PermanentEntered { card_id: c }]);
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life + 3, "gained life = number of Allies (3)");
}

#[test]
fn aven_eternal_amasses_zombies() {
    let mut g = two_player_game();
    let av = g.add_card_to_battlefield(0, catalog::aven_eternal());
    g.fire_self_etb_triggers(av, 0);
    drain_stack(&mut g);
    // No Army existed → a 0/0 Army token was made and given a +1/+1 counter.
    let army = g.battlefield.iter().find(|c| c.definition.name == "Army");
    assert!(army.is_some(), "made an Army token");
    assert_eq!(army.unwrap().counter_count(CounterType::PlusOnePlusOne), 1, "amass 1");
    assert!(army.unwrap().definition.subtypes.creature_types.contains(&crate::card::CreatureType::Zombie),
        "the Army is also a Zombie");
}

#[test]
fn storm_fleet_arsonist_sacrifices_only_after_attacking() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(1, catalog::grizzly_bears()); // opponent's only permanent
    // No attack this turn → raid off, no sacrifice.
    let a1 = g.add_card_to_battlefield(0, catalog::storm_fleet_arsonist());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Target(Target::Player(1))]));
    g.fire_self_etb_triggers(a1, 0);
    drain_stack(&mut g);
    assert_eq!(g.players[1].graveyard.len(), 0, "no raid → no sacrifice");
    // Mark that player 0 attacked, then a second Arsonist enters → sacrifice.
    g.players[0].attacked_this_turn = true;
    let a2 = g.add_card_to_battlefield(0, catalog::storm_fleet_arsonist());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Target(Target::Player(1))]));
    g.fire_self_etb_triggers(a2, 0);
    drain_stack(&mut g);
    assert_eq!(g.players[1].graveyard.len(), 1, "raid → opponent sacrificed a permanent");
}

#[test]
fn metallurgic_summonings_makes_xx_construct() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::metallurgic_summonings());
    // Cast a mana-value-3 instant (Divination is MV 3? use a known IS spell).
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt()); // MV 1
    g.players[0].mana_pool.add(crate::mana::Color::Red, 1);
    cast_sorcery(&mut g, 0, bolt, None);
    let con = g.battlefield.iter().find(|c| c.definition.name == "Construct").map(|c| c.id);
    assert!(con.is_some(), "cast an I/S → a Construct token");
    let cp = g.compute_battlefield();
    let c = cp.iter().find(|c| c.id == con.unwrap()).unwrap();
    assert_eq!((c.power, c.toughness), (1, 1), "X = the spell's mana value (Bolt = 1)");
}
