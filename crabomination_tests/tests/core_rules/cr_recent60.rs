//! CR conformance for this run's engine work:
//! - CR 601.2c — the Flagbearer targeting requirement: an opponent choosing
//!   targets must choose a Flagbearer if able.
//! - CR 701.9 — cycling is a discard, so it fires the "you discarded one or
//!   more cards" batch too.
//! - CR 115.9b — "a spell that targets you or a permanent you control" reads
//!   the spell's *current* targets.
//! - CR 603.2 — an "enchanted creature is dealt damage" trigger fires even
//!   when that damage was lethal.

use crabomination::catalog;
use crabomination::game::types::{GameAction, Target};
use crabomination::game::*;
use crabomination::mana::Color;

fn mana(g: &mut GameState, seat: usize) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[seat].mana_pool.add(c, 20);
    }
    g.players[seat].mana_pool.add_colorless(20);
}

fn main_phase() -> GameState {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g
}

/// CR 601.2c — with a Flagbearer up, a spell that could target it must.
#[test]
fn cr_601_2c_flagbearer_must_be_chosen_if_able() {
    let mut g = main_phase();
    g.add_card_to_battlefield(1, catalog::standard_bearer());
    let other = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    mana(&mut g, 0);
    assert!(
        g.perform_action(GameAction::CastSpell {
            card_id: bolt,
            target: Some(Target::Permanent(other)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .is_err(),
        "another creature isn't a legal choice while a Flagbearer is available"
    );
}

/// CR 601.2c — "if able": with no Flagbearer the spell could take, the
/// restriction imposes nothing.
#[test]
fn cr_601_2c_flagbearer_restriction_is_inert_when_unable() {
    let mut g = main_phase();
    g.add_card_to_battlefield(1, catalog::standard_bearer());
    let disenchant = g.add_card_to_hand(0, catalog::disenchant());
    let artifact = g.add_card_to_battlefield(1, catalog::sol_ring());
    mana(&mut g, 0);
    g.perform_action(GameAction::CastSpell {
        card_id: disenchant,
        target: Some(Target::Permanent(artifact)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("a creature Flagbearer can't be chosen by an artifact-only spell");
}

/// CR 601.2c — the auto-targeter satisfies the restriction on its own.
#[test]
fn cr_601_2c_auto_target_prefers_the_flagbearer() {
    let mut g = main_phase();
    let bearer = g.add_card_to_battlefield(1, catalog::standard_bearer());
    g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let effect = catalog::lightning_bolt().effect;
    assert_eq!(g.auto_target_for_effect(&effect, 0), Some(Target::Permanent(bearer)));
}

/// CR 701.9 — cycling counts as discarding one card for the batch trigger.
#[test]
fn cr_701_9_cycling_fires_the_discard_batch() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::magmakin_artillerist());
    let card = g.add_card_to_hand(0, catalog::secluded_steppe());
    g.add_card_to_library(0, catalog::plains());
    mana(&mut g, 0);
    g.perform_action(GameAction::Cycle { card_id: card, x_value: None }).expect("cycle");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 19, "the cycled card billed for 1");
}

/// CR 115.9b — the target-inspecting filter reads the spell's current targets.
#[test]
fn cr_115_9b_target_filter_reads_the_current_targets() {
    let mut g = main_phase();
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    mana(&mut g, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Permanent(theirs)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    let light = g.add_card_to_hand(0, catalog::hindering_light());
    mana(&mut g, 0);
    g.priority.player_with_priority = 0;
    assert!(
        g.perform_action(GameAction::CastSpell {
            card_id: light,
            target: Some(Target::Permanent(bolt)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .is_err(),
        "the Bolt targets their own creature, not mine"
    );
    let _ = mine;
}

/// CR 603.2 — the trigger condition is checked before the lethal-damage SBA,
/// so an Aura's "enchanted creature is dealt damage" watcher still fires.
#[test]
fn cr_603_2_damage_trigger_survives_a_lethal_hit() {
    let mut g = main_phase();
    let bears = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let link = g.add_card_to_hand(0, catalog::soul_link());
    mana(&mut g, 0);
    g.perform_action(GameAction::CastSpell {
        card_id: link,
        target: Some(Target::Permanent(bears)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    mana(&mut g, 0);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Permanent(bears)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().all(|c| c.id != bears), "3 damage killed the 2/2");
    assert_eq!(g.players[0].life, 23, "Soul Link still paid out");
}
