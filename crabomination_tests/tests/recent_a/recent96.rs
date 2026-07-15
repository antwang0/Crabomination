//! Functionality tests for the `catalog::sets::decks::recent96` Kamigawa: Neon
//! Dynasty batch 2.

use crabomination::catalog;
use crabomination::card::{CounterType, Keyword};
use crabomination::game::two_player_game;
use crabomination::game::*;

/// Jukai Naturalist discounts enchantment spells by {1}.
#[test]
fn jukai_naturalist_discounts_enchantments() {
    use crabomination::game::actions::cost_reduction_for_spell;
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::jukai_naturalist());
    let ench = crabomination::card::CardInstance::new(g.next_id(), catalog::golden_tail_disciple(), 0);
    let creature = crabomination::card::CardInstance::new(g.next_id(), catalog::grizzly_bears(), 0);
    assert_eq!(cost_reduction_for_spell(&g, 0, &ench, None), 1, "enchantment −{{1}}");
    assert_eq!(cost_reduction_for_spell(&g, 0, &creature, None), 0, "plain creature unaffected");
}

/// Kami of Transience grows when you cast an enchantment spell.
#[test]
fn kami_of_transience_grows_on_enchantment_cast() {
    let mut g = two_player_game();
    let kami = g.add_card_to_battlefield(0, catalog::kami_of_transience());
    let aura = g.add_card_to_hand(0, catalog::golden_tail_disciple()); // enchantment creature
    for _ in 0..2 { g.players[0].mana_pool.add(crabomination::mana::Color::White, 1); }
    g.players[0].mana_pool.add_colorless(1);
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::CastSpell {
        card_id: aura, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast an enchantment");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(kami).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
}

/// Rabbit Battery grants +1/+1 and haste when attached.
#[test]
fn rabbit_battery_equip_bonus() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let battery = g.add_card_to_battlefield(0, catalog::rabbit_battery());
    g.battlefield.iter_mut().find(|c| c.id == battery).unwrap().attached_to = Some(bear);
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 3), "2/2 + 1/1");
    assert!(cp.keywords.contains(&Keyword::Haste));
}

/// Nezumi Prowler's ETB grants deathtouch and lifelink to a creature you control.
#[test]
fn nezumi_prowler_etb_grants_keywords() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let nezumi = g.add_card_to_battlefield(0, catalog::nezumi_prowler());
    // Retarget the auto-picked creature onto the bear by removing other options:
    // both bear and nezumi are legal; assert the grant landed on one of them.
    g.fire_self_etb_triggers(nezumi, 0);
    drain_stack(&mut g);
    let got = [bear, nezumi].iter().any(|id| {
        let kw = &g.computed_permanent(*id).unwrap().keywords;
        kw.contains(&Keyword::Deathtouch) && kw.contains(&Keyword::Lifelink)
    });
    assert!(got, "a creature you control gained deathtouch + lifelink");
}

/// Invigorating Hot Spring enters with four counters and grants haste to
/// modified creatures.
#[test]
fn invigorating_hot_spring_hastes_modified() {
    let mut g = two_player_game();
    let spring = g.move_card_to_battlefield_for_test(0, catalog::invigorating_hot_spring());
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(spring).unwrap().counter_count(CounterType::PlusOnePlusOne), 4,
        "entered with four +1/+1 counters");
    // A bear equipped with Bonesplitter is "modified" → gains haste.
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let axe = g.add_card_to_battlefield(0, catalog::bonesplitter());
    g.battlefield.iter_mut().find(|c| c.id == axe).unwrap().attached_to = Some(bear);
    assert!(g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Haste),
        "modified creature has haste");
}

/// Ironhoof Boar's Channel ability pumps a creature +3/+1 with trample.
#[test]
fn ironhoof_boar_channel_pumps() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let boar = g.add_card_to_hand(0, catalog::ironhoof_boar());
    g.players[0].mana_pool.add(crabomination::mana::Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    // The Channel ability is the from-hand activated ability (index 0).
    g.perform_action(GameAction::ActivateAbility {
        card_id: boar, ability_index: 0, target: Some(crabomination::game::types::Target::Permanent(bear)),
        additional_targets: Vec::new(), x_value: None,
    }).expect("channel the Boar");
    drain_stack(&mut g);
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!((cp.power, cp.toughness), (5, 3), "2/2 + 3/1");
    assert!(cp.keywords.contains(&Keyword::Trample));
    assert!(g.players[0].graveyard.iter().any(|c| c.definition.name == "Ironhoof Boar"),
        "the Boar was discarded to pay Channel");
}

/// Reinforced Ronin's Channel ability draws a card from hand.
#[test]
fn reinforced_ronin_channel_draws() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let ronin = g.add_card_to_hand(0, catalog::reinforced_ronin());
    g.players[0].mana_pool.add(crabomination::mana::Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::ActivateAbility {
        card_id: ronin, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("channel: draw");
    drain_stack(&mut g);
    // −1 (Ronin discarded) +1 (drew) = net 0, and the Ronin is in the graveyard.
    assert_eq!(g.players[0].hand.len(), hand_before, "discarded Ronin, drew a card");
    assert!(g.players[0].graveyard.iter().any(|c| c.definition.name == "Reinforced Ronin"));
}

/// Colossal Skyturtle's blue Channel bounces a creature to its owner's hand.
#[test]
fn colossal_skyturtle_channel_bounces() {
    let mut g = two_player_game();
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let turtle = g.add_card_to_hand(0, catalog::colossal_skyturtle());
    g.players[0].mana_pool.add(crabomination::mana::Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    // The blue Channel is the second from-hand ability (index 1).
    g.perform_action(GameAction::ActivateAbility {
        card_id: turtle, ability_index: 1, target: Some(crabomination::game::types::Target::Permanent(foe)),
        additional_targets: Vec::new(), x_value: None,
    }).expect("channel: bounce");
    drain_stack(&mut g);
    assert!(g.battlefield_find(foe).is_none(), "bounced");
    assert!(g.players[1].hand.iter().any(|c| c.definition.name == "Grizzly Bears"));
}
