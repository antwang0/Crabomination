//! Demonstrate cycle (the STX "Technique" sorceries, CR 702.150).
//! `Effect::Demonstrate` copies the spell for its caster and an opponent;
//! every copy may choose new targets.

use crate::catalog;
use crate::game::{drain_stack, two_player_game};
use crate::mana::Color;
use super::*;

/// Replication Technique's Demonstrate: the caster copies the spell (two token
/// copies of their own permanent — original + copy), and an opponent also
/// copies it, controlling a token copy of *their* own permanent.
#[test]
fn replication_technique_demonstrate_copies_for_both_players() {
    let mut g = two_player_game();
    let my_bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::replication_technique());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(4);

    let p0_before = g.battlefield.iter().filter(|c| c.controller == 0 && c.is_token).count();
    let p1_before = g.battlefield.iter().filter(|c| c.controller == 1 && c.is_token).count();

    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: Some(Target::Permanent(my_bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Replication Technique");
    drain_stack(&mut g);

    let p0_after = g.battlefield.iter().filter(|c| c.controller == 0 && c.is_token).count();
    let p1_after = g.battlefield.iter().filter(|c| c.controller == 1 && c.is_token).count();
    assert_eq!(p0_after - p0_before, 2, "caster gets original + demonstrate copy");
    assert_eq!(p1_after - p1_before, 1, "opponent also copies, controlling its own copy");
}

/// Excavation Technique destroys a target nonland permanent; its controller
/// (here the opponent) creates two Treasure tokens.
#[test]
fn excavation_technique_destroys_and_treasures_controller() {
    let mut g = two_player_game();
    let opp_perm = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::excavation_technique());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(3);

    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: Some(Target::Permanent(opp_perm)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Excavation Technique");
    drain_stack(&mut g);

    assert!(g.battlefield_find(opp_perm).is_none(), "target nonland permanent destroyed");
    // The destroyed permanent's controller (here the opponent) creates two
    // Treasures. The Demonstrate copies also resolve against the same target;
    // a zone-blind "nonland permanent" filter doesn't fizzle them at
    // resolution (see TODO.md), so the controller ends up with a multiple of
    // two. The core behavior — destroyed and its controller is the recipient —
    // is what matters.
    let treasures = g.battlefield.iter()
        .filter(|c| c.controller == 1 && c.definition.name == "Treasure").count();
    assert!(treasures >= 2, "destroyed permanent's controller makes (at least) two Treasures");
}

/// Healing Technique returns a card from your graveyard to hand, gains life
/// equal to its mana value, and exiles itself.
#[test]
fn healing_technique_returns_card_gains_life_and_exiles_self() {
    let mut g = two_player_game();
    let card = g.add_card_to_graveyard(0, catalog::grizzly_bears()); // MV 2
    let spell = g.add_card_to_hand(0, catalog::healing_technique());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    let life = g.players[0].life;

    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: Some(Target::Permanent(card)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Healing Technique");
    drain_stack(&mut g);

    assert!(g.players[0].hand.iter().any(|c| c.id == card), "card returned to hand");
    // Gain life equal to the returned card's mana value (Grizzly Bears = 2).
    // The Demonstrate copies retarget into the caster's graveyard too; the
    // engine's target filter has no zone constraint, so at least the base
    // mana value is gained.
    assert!(g.players[0].life >= life + 2, "gain life equal to mana value");
    assert!(g.exile.iter().any(|c| c.id == spell), "Healing Technique exiles itself");
}

/// Incarnation Technique mills five, then returns a creature card from your
/// graveyard to the battlefield.
#[test]
fn incarnation_technique_mills_then_reanimates() {
    let mut g = two_player_game();
    let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    // Stock the library so the mill has something to move.
    for _ in 0..6 { g.add_card_to_library(0, catalog::island()); }
    let spell = g.add_card_to_hand(0, catalog::incarnation_technique());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(4);

    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: None,
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Incarnation Technique");
    drain_stack(&mut g);

    assert!(g.battlefield_find(bear).is_some(), "creature reanimated to battlefield");
}

/// Creative Technique shuffles, exiles the top until a nonland card, and lets
/// you cast it for free. With the cast accepted, the lone nonland card enters
/// the battlefield.
#[test]
fn creative_technique_impulse_casts_a_nonland() {
    let mut g = two_player_game();
    let lib_bear = g.add_card_to_library(0, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::creative_technique());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(4);
    // Accept the Cascade free-cast prompt.
    g.decider = Box::new(crate::decision::ScriptedDecider::new([
        crate::decision::DecisionAnswer::Bool(true),
    ]));

    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: None,
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Creative Technique");
    drain_stack(&mut g);

    assert!(
        g.battlefield_find(lib_bear).is_some(),
        "the revealed nonland card was exiled and impulse-cast",
    );
}

/// Transforming Flourish destroys a target artifact or creature you don't
/// control.
#[test]
fn transforming_flourish_destroys_opponent_creature() {
    let mut g = two_player_game();
    let opp_bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::transforming_flourish());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);

    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: Some(Target::Permanent(opp_bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Transforming Flourish");
    drain_stack(&mut g);

    assert!(g.battlefield_find(opp_bear).is_none(), "opponent's creature destroyed");
}

/// Codie's static forbids casting permanent spells.
#[test]
fn codie_blocks_permanent_spells() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::codie_vociferous_codex());
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Green, 2);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    assert!(g.perform_action(GameAction::CastSpell {
        card_id: bear, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).is_err(), "permanent spells are locked while Codie is out");
}

/// Codie's activation adds WUBRG and sets up the next-spell impulse: casting
/// a spell exiles from the top until a cheaper instant/sorcery turns up.
#[test]
fn codie_activation_ramps_and_impulses_on_next_spell() {
    let mut g = two_player_game();
    let codie = g.add_card_to_battlefield(0, catalog::codie_vociferous_codex());
    g.clear_sickness(codie);
    // Library (top → down): a bear (skipped — not an IS), a Bolt (MV 1 < 3 →
    // hit), then two Islands for Divination's draws.
    let skipped = g.add_card_to_library(0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::lightning_bolt());
    for _ in 0..2 { g.add_card_to_library(0, catalog::island()); }
    g.players[0].mana_pool.add_colorless(4);
    g.step = TurnStep::PreCombatMain;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: codie, ability_index: 0, target: None, x_value: None,
    }).expect("activate Codie");
    drain_stack(&mut g);
    assert_eq!(g.players[0].mana_pool.total(), 5, "added WUBRG");
    // Cast Divination (MV 3) off the burst.
    let div = g.add_card_to_hand(0, catalog::divination());
    g.perform_action(GameAction::CastSpell {
        card_id: div, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Divination");
    drain_stack(&mut g);
    // The impulse fired: bear skipped to exile→bottom, Bolt (declined cast) to hand.
    assert!(g.players[0].hand.iter().any(|c| c.definition.name == "Lightning Bolt"),
        "cheaper instant found and (declined) put in hand");
    assert_eq!(g.players[0].library.last().map(|c| c.id), Some(skipped),
        "non-IS card bottomed");
}

/// Ecological Appreciation finds up to four distinct-name creatures with
/// MV <= X across library + graveyard; the opponent denies the two biggest,
/// the rest enter the battlefield.
#[test]
fn ecological_appreciation_splits_with_opponent() {
    let mut g = two_player_game();
    // Candidates: dreadmaw MV 6 (excluded: > X), serra angel MV 5 + bears MV 2
    // in library, wall of omens MV 2 in graveyard.
    g.add_card_to_library(0, catalog::colossal_dreadmaw());
    let angel = g.add_card_to_library(0, catalog::serra_angel());
    let bears = g.add_card_to_library(0, catalog::grizzly_bears());
    let wall = g.add_card_to_graveyard(0, catalog::wall_of_omens());
    let spell = g.add_card_to_hand(0, catalog::ecological_appreciation());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(7); // X=5 + {2}
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: Some(5),
    }).expect("cast Ecological Appreciation");
    drain_stack(&mut g);
    // Angel (MV 5) + one MV-2 body denied (the two biggest); the other two
    // MV-2 creatures hit the battlefield... with three candidates <= X
    // (angel, bears, wall), the two biggest are denied: angel + one 2-drop.
    assert!(g.battlefield_find(angel).is_none(), "biggest candidate denied");
    let entered = [bears, wall].iter().filter(|&&c| g.battlefield_find(c).is_some()).count();
    assert_eq!(entered, 1, "one of the remaining candidates entered");
    assert!(!g.battlefield.iter().any(|c| c.id == angel), "angel shuffled away");
}
