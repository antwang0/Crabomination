//! Functionality tests for `catalog::sets::decks::recent236` (DSK Rooms +
//! Terror of Towashi).

use crabomination::card::{CardType, CounterType, Keyword};
use crabomination::catalog;
use crabomination::effect::Effect;
use crabomination::game::effects::EffectContext;
use crabomination::game::types::Target;
use crabomination::game::{drain_stack, two_player_game};

fn door_effect(def: &crabomination::card::CardDefinition, right: bool) -> Effect {
    let room = def.room.as_ref().expect("room card");
    let door = if right { &room.right } else { &room.left };
    door.triggered_abilities[0].effect.clone()
}

/// Grand Entryway's unlock makes a 1/1 Glimmer enchantment creature.
#[test]
fn grand_entryway_makes_glimmer() {
    let mut g = two_player_game();
    let def = catalog::grand_entryway_elegant_rotunda();
    let src = g.add_card_to_battlefield(0, catalog::grand_entryway_elegant_rotunda());
    let ctx = EffectContext::for_trigger(src, 0, None, 0);
    g.resolve_effect(&door_effect(&def, false), &ctx).unwrap();
    let glimmer = g
        .battlefield
        .iter()
        .find(|c| c.definition.name == "Glimmer" && c.controller == 0)
        .expect("Glimmer token");
    assert!(glimmer.definition.card_types.contains(&CardType::Enchantment), "enchantment creature");
}

/// Elegant Rotunda puts a +1/+1 counter on each of up to two target creatures.
#[test]
fn elegant_rotunda_counters_two() {
    let mut g = two_player_game();
    let a = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let def = catalog::grand_entryway_elegant_rotunda();
    let src = g.add_card_to_battlefield(0, catalog::grand_entryway_elegant_rotunda());
    let ctx = EffectContext {
        targets: vec![Target::Permanent(a), Target::Permanent(b)],
        ..EffectContext::for_trigger(src, 0, None, 0)
    };
    g.resolve_effect(&door_effect(&def, true), &ctx).unwrap();
    assert_eq!(g.battlefield_find(a).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
    assert_eq!(g.battlefield_find(b).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
}

/// Derelict Attic draws two and loses 2 life on unlock.
#[test]
fn derelict_attic_draw_lose() {
    let mut g = two_player_game();
    for _ in 0..2 {
        g.add_card_to_library(0, catalog::grizzly_bears());
    }
    let def = catalog::derelict_attic_widows_walk();
    let src = g.add_card_to_battlefield(0, catalog::derelict_attic_widows_walk());
    let hand = g.players[0].hand.len();
    let life = g.players[0].life;
    let ctx = EffectContext::for_trigger(src, 0, None, 0);
    g.resolve_effect(&door_effect(&def, false), &ctx).unwrap();
    assert_eq!(g.players[0].hand.len(), hand + 2, "drew two");
    assert_eq!(g.players[0].life, life - 2, "lost 2 life");
}

/// Funeral Room drains 1 whenever a creature you control dies.
#[test]
fn funeral_room_drains_on_death() {
    let mut g = two_player_game();
    let room = g.add_card_to_battlefield(0, catalog::funeral_room_awakening_hall());
    // The Funeral Room (left) door's death drain is live only once unlocked.
    g.battlefield_find_mut(room).unwrap().unlock_room_door(false);
    let victim = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let opp_life = g.players[1].life;
    let my_life = g.players[0].life;
    let mut evs = g.remove_to_graveyard_with_triggers(victim);
    evs.push(crabomination::game::GameEvent::CreatureDied { card_id: victim });
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_life - 1, "opponent lost 1");
    assert_eq!(g.players[0].life, my_life + 1, "you gained 1");
}

/// Awakening Hall returns every creature card from your graveyard.
#[test]
fn awakening_hall_mass_reanimate() {
    let mut g = two_player_game();
    let a = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let b = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let def = catalog::funeral_room_awakening_hall();
    let src = g.add_card_to_battlefield(0, catalog::funeral_room_awakening_hall());
    let ctx = EffectContext::for_trigger(src, 0, None, 0);
    g.resolve_effect(&door_effect(&def, true), &ctx).unwrap();
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.id == a), "first creature reanimated");
    assert!(g.battlefield.iter().any(|c| c.id == b), "second creature reanimated");
}

/// Defaced Gallery pumps attacking creatures when you attack.
#[test]
fn defaced_gallery_pumps_attackers() {
    let mut g = two_player_game();
    let def = catalog::painters_studio_defaced_gallery();
    let src = g.add_card_to_battlefield(0, catalog::painters_studio_defaced_gallery());
    let attacker = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.set_attacking(vec![crabomination::game::types::Attack {
        attacker,
        target: crabomination::game::types::AttackTarget::Player(1),
    }]);
    let ctx = EffectContext::for_trigger(src, 0, None, 0);
    g.resolve_effect(&door_effect(&def, true), &ctx).unwrap();
    assert_eq!(g.computed_permanent(attacker).unwrap().power, 3, "2 + 1 = 3");
}

/// Terror of Towashi's attack rider reanimates when the cost is paid.
#[test]
fn terror_of_towashi_reanimates_on_pay() {
    let mut g = two_player_game();
    let dead = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let terror = g.add_card_to_battlefield(0, catalog::terror_of_towashi());
    g.players[0].mana_pool.add(crabomination::mana::Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);
    // Say yes to the {3}{B} may-pay, then target the graveyard creature.
    g.decider = Box::new(crabomination::decision::ScriptedDecider::new([
        crabomination::decision::DecisionAnswer::Bool(true),
    ]));
    let effect = catalog::terror_of_towashi().triggered_abilities[0].effect.clone();
    let ctx = EffectContext::for_trigger(terror, 0, None, 0);
    g.resolve_effect(&effect, &ctx).unwrap();
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.id == dead), "creature reanimated after paying");
    // Deathtouch is printed.
    assert!(catalog::terror_of_towashi().keywords.contains(&Keyword::Deathtouch));
}
