//! Functionality tests for `catalog::sets::decks::recent142` (WOE legends/spells).

use crate::catalog;
use crate::card::CounterType;
use crate::game::types::Target;
use crate::game::*;
use crate::game::{drain_stack, two_player_game};
use crate::mana::Color;

fn cast(g: &mut GameState, id: CardId, target: Option<Target>, x: Option<u32>) {
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target,
        additional_targets: vec![],
        mode: None,
        x_value: x,
    })
    .expect("cast");
    drain_stack(g);
}

/// Greta enters with a Food and can sacrifice it to grow a creature.
#[test]
fn greta_food_and_counter() {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let id = g.add_card_to_hand(0, catalog::greta_sweettooth_scourge());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    cast(&mut g, id, None, None);
    let greta = g.battlefield.iter().find(|c| c.definition.name.starts_with("Greta")).unwrap().id;
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Food"), "made a Food");
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: greta,
        ability_index: 0,
        target: Some(Target::Permanent(greta)),
        additional_targets: vec![],
        x_value: None,
    })
    .expect("sac a Food for a counter");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(greta).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
    assert!(!g.battlefield.iter().any(|c| c.definition.name == "Food"), "Food sacrificed");
}

/// Totentanz makes a Rat when a nontoken creature you control dies.
#[test]
fn totentanz_rat_on_death() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::totentanz_swarm_piper());
    let victim = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(victim).unwrap().damage = 2; // lethal → CreatureDied
    let evs = g.check_state_based_actions();
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Rat"), "Rat minted on death");
}

/// Neva returns an enchantment card from your graveyard on entry, and grows +
/// scries when an enchantment you control dies.
#[test]
fn neva_returns_and_grows() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let ench = g.add_card_to_graveyard(0, catalog::a_tale_for_the_ages());
    let id = g.add_card_to_hand(0, catalog::neva_stalked_by_nightmares());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.add_card_to_library(0, catalog::grizzly_bears()); // for the scry
    cast(&mut g, id, Some(Target::Permanent(ench)), None);
    assert!(g.players[0].hand.iter().any(|c| c.id == ench), "enchantment returned to hand");
    let neva = g.battlefield.iter().find(|c| c.definition.name.starts_with("Neva")).unwrap().id;
    let onbf = g.add_card_to_battlefield(0, catalog::a_tale_for_the_ages());
    let ctx = crate::game::effects::EffectContext::for_ability(onbf, 0, None);
    let evs = g.resolve_effect(
        &crate::effect::Effect::Destroy { what: crate::effect::Selector::This },
        &ctx,
    )
    .unwrap();
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(neva).unwrap().counter_count(CounterType::PlusOnePlusOne), 1, "Neva grew");
}

/// Syr Armont hangs a Monster Role and his anthem stacks: a 2/2 becomes 4/4.
#[test]
fn syr_armont_role_and_anthem() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::syr_armont_the_redeemer());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(3);
    cast(&mut g, id, Some(Target::Permanent(bear)), None);
    let b = g.computed_permanent(bear).unwrap();
    assert_eq!((b.power, b.toughness), (4, 4), "+1/+1 from Role and +1/+1 from anthem");
}

/// Troyan's restricted mana funds an expensive spell but not a cheap one.
#[test]
fn troyan_high_mv_or_x_restriction() {
    use crate::mana::{SpellKind, SpendRestriction};
    let r = SpendRestriction::HighMvOrX;
    let big = SpellKind { mana_value: 6, ..Default::default() };
    let xspell = SpellKind { mana_value: 1, has_x: true, ..Default::default() };
    let cheap = SpellKind { mana_value: 3, ..Default::default() };
    assert!(r.allows(&big), "funds a mana value 5+ spell");
    assert!(r.allows(&xspell), "funds an {{X}} spell");
    assert!(!r.allows(&cheap), "not a small non-X spell");
    // The ability floats restricted mana into the pool.
    let mut g = two_player_game();
    let troyan = g.add_card_to_battlefield(0, catalog::troyan_gutsy_explorer());
    g.battlefield_find_mut(troyan).unwrap().summoning_sick = false;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: troyan,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
    })
    .expect("tap for restricted mana");
    assert_eq!(g.players[0].mana_pool.restricted_total(), 2, "two restricted mana floated");
}

/// Johann lets you cast one instant/sorcery from the top of your library each
/// turn; the second top card can't be cast until the cap resets.
#[test]
fn johann_casts_from_top_once_per_turn() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::johann_apprentice_sorcerer());
    let bolt1 = g.add_card_to_library(0, catalog::lightning_bolt()); // top of library
    let bolt2 = g.add_card_to_library(0, catalog::lightning_bolt());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Red, 2);
    cast(&mut g, bolt1, Some(Target::Player(1)), None);
    assert_eq!(g.players[1].life, 17, "first Bolt cast from top dealt 3");
    assert!(
        g.perform_action(GameAction::CastSpell {
            card_id: bolt2,
            target: Some(Target::Player(1)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .is_err(),
        "second top-of-library cast is blocked by the once-per-turn cap",
    );
    assert_eq!(g.players[1].life, 17, "no second Bolt this turn");
}

/// Solitary Sanctuary grows one of your creatures when you tap an enemy creature.
#[test]
fn solitary_sanctuary_tap_grows() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let enemy = g.add_card_to_battlefield(1, catalog::serra_angel());
    let id = g.add_card_to_hand(0, catalog::solitary_sanctuary());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    cast(&mut g, id, Some(Target::Permanent(enemy)), None);
    assert!(g.battlefield_find(enemy).unwrap().tapped, "enemy tapped by ETB");
    // The ETB tap is itself a "you tap …" event, so your creature grows.
    assert_eq!(g.battlefield_find(mine).unwrap().counter_count(CounterType::PlusOnePlusOne), 1, "your creature grew");
}

/// Farsight Ritual digs four and pulls two into hand.
#[test]
fn farsight_ritual_digs_two() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    for _ in 0..4 { g.add_card_to_library(0, catalog::grizzly_bears()); }
    let id = g.add_card_to_hand(0, catalog::farsight_ritual());
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(2);
    let hand = g.players[0].hand.len();
    cast(&mut g, id, None, None);
    // Ritual leaves hand (−1), two cards drawn (+2) → net +1.
    assert_eq!(g.players[0].hand.len(), hand + 1, "two cards taken to hand");
}
