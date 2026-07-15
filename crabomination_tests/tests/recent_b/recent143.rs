//! Functionality tests for `catalog::sets::decks::recent143` (WOE Adventures).

use crabomination::catalog;
use crabomination::game::types::Target;
use crabomination::game::*;
use crabomination::game::{drain_stack, two_player_game};
use crabomination::mana::Color;

fn cast(g: &mut GameState, id: CardId, target: Option<Target>) {
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(g);
}

fn cast_adventure(g: &mut GameState, id: CardId, target: Option<Target>) {
    g.perform_action(GameAction::CastAdventure {
        card_id: id,
        target,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast adventure");
    drain_stack(g);
}

/// Ferocious Werefox's Guard Change adventure hangs a Monster Role (+1/+1 trample).
#[test]
fn guard_change_hangs_monster_role() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::ferocious_werefox());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    cast_adventure(&mut g, id, Some(Target::Permanent(bear)));
    let b = g.computed_permanent(bear).unwrap();
    assert_eq!(b.power, 3, "2/2 + Monster Role +1/+1");
    assert!(b.keywords.contains(&crabomination::card::Keyword::Trample), "Role grants trample");
}

/// Hare Raising pumps a creature by the number of creatures you control.
#[test]
fn hare_raising_scales_with_board() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let a = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 3 creatures
    let id = g.add_card_to_hand(0, catalog::pollen_shield_hare());
    g.players[0].mana_pool.add(Color::Green, 1);
    cast_adventure(&mut g, id, Some(Target::Permanent(a)));
    let c = g.computed_permanent(a).unwrap();
    assert_eq!(c.power, 5, "2 + 3 creatures");
    assert!(c.keywords.contains(&crabomination::card::Keyword::Vigilance), "gains vigilance");
}

/// Frolicking Familiar grows when you cast an instant.
#[test]
fn frolicking_familiar_grows_on_instant() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let fam = g.add_card_to_battlefield(0, catalog::frolicking_familiar());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    cast(&mut g, bolt, Some(Target::Player(1)));
    assert_eq!(g.computed_permanent(fam).unwrap().power, 3, "+1/+1 on instant cast");
}

/// Gumdrop Poisoner's ETB shrinks a creature by the life you gained this turn.
#[test]
fn gumdrop_poisoner_minus_x() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].life_gained_this_turn = 3;
    let victim = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4
    let id = g.add_card_to_hand(0, catalog::gumdrop_poisoner());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    cast(&mut g, id, Some(Target::Permanent(victim)));
    assert_eq!(g.computed_permanent(victim).unwrap().power, 1, "-3/-3 from 3 life gained");
}

/// Croaking Curse taps a creature and makes it 1/1 with a Cursed Role.
#[test]
fn croaking_curse_taps_and_shrinks() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let enemy = g.add_card_to_battlefield(1, catalog::serra_angel());
    let id = g.add_card_to_hand(0, catalog::vantress_transmuter());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    cast_adventure(&mut g, id, Some(Target::Permanent(enemy)));
    assert!(g.battlefield_find(enemy).unwrap().tapped, "tapped");
    assert_eq!(g.computed_permanent(enemy).unwrap().power, 1, "Cursed Role sets it to 1/1");
}

/// Free the Fae mills four and takes an instant/sorcery/Faerie to hand.
#[test]
fn free_the_fae_mills_and_takes() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let bolt = g.add_card_to_library(0, catalog::lightning_bolt()); // an instant among the top four
    for _ in 0..3 { g.add_card_to_library(0, catalog::grizzly_bears()); }
    let id = g.add_card_to_hand(0, catalog::picklock_prankster());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.decider = Box::new(crabomination::decision::ScriptedDecider::new([
        crabomination::decision::DecisionAnswer::Cards(vec![bolt]),
    ]));
    cast_adventure(&mut g, id, None);
    assert!(g.players[0].hand.iter().any(|c| c.definition.name == "Lightning Bolt"), "took the instant");
}

/// Bear Down destroys an artifact or enchantment.
#[test]
fn bear_down_destroys_enchantment() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let ench = g.add_card_to_battlefield(1, catalog::a_tale_for_the_ages());
    let id = g.add_card_to_hand(0, catalog::stormkeld_vanguard());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    cast_adventure(&mut g, id, Some(Target::Permanent(ench)));
    assert!(g.battlefield_find(ench).is_none(), "enchantment destroyed");
}

/// Scalding Viper pings an opponent who casts a cheap spell.
#[test]
fn scalding_viper_pings_cheap_caster() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    g.add_card_to_battlefield(0, catalog::scalding_viper());
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt()); // MV 1
    g.players[1].mana_pool.add(Color::Red, 1);
    g.priority.player_with_priority = 1;
    let life = g.players[1].life;
    cast(&mut g, bolt, Some(Target::Player(0)));
    assert_eq!(g.players[1].life, life - 1, "opponent pinged for casting a MV≤3 spell");
}
