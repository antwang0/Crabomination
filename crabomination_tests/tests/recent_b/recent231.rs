//! Functionality tests for `catalog::sets::decks::recent231`.

use crabomination::card::CounterType;
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::effects::EffectContext;
use crabomination::game::types::Target;
use crabomination::game::{drain_stack, two_player_game};

/// Volcanic Spite deals 3 to a creature; the optional loot is declined cleanly.
#[test]
fn volcanic_spite_burns_a_creature() {
    let mut g = two_player_game();
    let enemy = g.add_card_to_battlefield(1, catalog::hill_giant()); // 3/3
    let ctx = EffectContext {
        targets: vec![Target::Permanent(enemy)],
        ..EffectContext::for_spell(0, None, 0, 0)
    };
    // AutoDecider declines the optional bottom-then-draw.
    g.resolve_effect(&catalog::volcanic_spite().effect.clone(), &ctx).unwrap();
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == enemy), "3/3 dies to 3 damage");
}

/// Volcanic Spite's opt-in loot bottoms a card and draws a replacement.
#[test]
fn volcanic_spite_loots_when_accepted() {
    let mut g = two_player_game();
    let enemy = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.add_card_to_hand(0, catalog::forest());
    g.players[0].add_to_library_top(crabomination::card::CardId(9100), catalog::mountain());
    let hand_before = g.players[0].hand.len();
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let ctx = EffectContext {
        targets: vec![Target::Permanent(enemy)],
        ..EffectContext::for_spell(0, None, 0, 0)
    };
    g.resolve_effect(&catalog::volcanic_spite().effect.clone(), &ctx).unwrap();
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand_before, "bottomed one, drew one");
}

/// Rampaging Soulrager is 1/4 with no unlocked doors and 4/4 once two doors
/// among your Rooms are unlocked.
#[test]
fn rampaging_soulrager_grows_with_two_doors() {
    let mut g = two_player_game();
    let sr = g.add_card_to_battlefield(0, catalog::rampaging_soulrager());
    assert_eq!(g.computed_permanent(sr).unwrap().power, 1, "1/4 with no doors");
    let room = g.add_card_to_battlefield(0, catalog::roaring_furnace_steaming_sauna());
    g.battlefield_find_mut(room).unwrap().unlock_room_door(false);
    g.battlefield_find_mut(room).unwrap().unlock_room_door(true);
    assert_eq!(g.computed_permanent(sr).unwrap().power, 4, "+3/+0 with two unlocked doors");
}

/// Lilysplash Mentor blinks your creature and returns it with a +1/+1 counter.
#[test]
fn lilysplash_mentor_blinks_with_counter() {
    let mut g = two_player_game();
    let mentor = g.add_card_to_battlefield(0, catalog::lilysplash_mentor());
    let ally = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let effect = catalog::lilysplash_mentor().activated_abilities[0].effect.clone();
    let ctx = EffectContext {
        targets: vec![Target::Permanent(ally)],
        ..EffectContext::for_ability(mentor, 0, None)
    };
    g.resolve_effect(&effect, &ctx).unwrap();
    drain_stack(&mut g);
    // The original object left; a fresh Grizzly Bears is back with a counter.
    let returned = g
        .battlefield
        .iter()
        .find(|c| c.controller == 0 && c.definition.name == "Grizzly Bears")
        .expect("creature returned to the battlefield");
    assert_eq!(returned.counter_count(CounterType::PlusOnePlusOne), 1, "returns with a +1/+1 counter");
}
