//! CR conformance for this run:
//! - CR 206.3a/b/c — "a name originally printed in [set]" checks a fixed name
//!   list, not a live expansion symbol.
//! - CR 702.177a — an exhaust ability is "activate only once", and CR 400.7's
//!   new object gets a fresh one.
//! - CR 111.7 / 111.8 — a token that leaves the battlefield ceases to exist and
//!   can't come back.

use crabomination::card::{CardId, OriginalSet};
use crabomination::catalog;
use crabomination::effect::{Effect, PlayerRef, Selector, Value, ZoneDest};
use crabomination::game::effects::EffectContext;
use crabomination::game::types::{GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn mana(g: &mut GameState, seat: usize) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[seat].mana_pool.add(c, 20);
    }
    g.players[seat].mana_pool.add_colorless(20);
}

/// CR 206.3a/b/c — each set's name list is the whole test; an unrelated card is
/// out, and the three lists don't leak into each other.
#[test]
fn cr_206_3_original_set_names_are_matched_by_name() {
    assert!(OriginalSet::ArabianNights.contains("Library of Alexandria"));
    assert!(OriginalSet::Antiquities.contains("Mishra's Factory"));
    assert!(OriginalSet::Homelands.contains("Memory Lapse"));
    assert!(!OriginalSet::Antiquities.contains("Library of Alexandria"));
    assert!(!OriginalSet::ArabianNights.contains("Lightning Bolt"));
    assert!(!OriginalSet::Homelands.contains("Mishra's Factory"));
}

/// CR 702.177a — "Activate only once": the second activation is rejected, and
/// CR 400.7's new object gets a fresh one.
#[test]
fn cr_702_177a_exhaust_is_once_per_object() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    let sita = g.add_card_to_battlefield(0, catalog::sita_varma_masked_racer());
    g.clear_sickness(sita);
    let activate = |g: &mut GameState, id: CardId| {
        mana(g, 0);
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::ActivateAbility {
            card_id: id,
            ability_index: 0,
            target: None,
            additional_targets: vec![],
            x_value: Some(1),
            mode: None,
        })
    };
    activate(&mut g, sita).expect("first activation");
    drain_stack(&mut g);
    assert!(activate(&mut g, sita).is_err(), "exhaust is once only");

    // A blink makes a new object, so the exhaust ability is available again.
    let ctx = EffectContext::for_spell(0, None, 0, 0);
    let mut events = vec![];
    g.move_card_to(sita, &ZoneDest::Exile, &ctx, &mut events);
    g.move_card_to(
        sita,
        &ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
        &ctx,
        &mut events,
    );
    g.clear_sickness(sita);
    activate(&mut g, sita).expect("a new object has a fresh exhaust");
}

/// CR 111.7 / 111.8 — a token that dies ceases to exist, and putting it back is
/// a no-op rather than a resurrection.
#[test]
fn cr_111_8_a_dead_token_cant_come_back() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    let ctx = EffectContext::for_spell(0, None, 0, 0);
    g.resolve_effect(
        &Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::ONE,
            definition: crabomination::game::effects::treasure_token(),
        },
        &ctx,
    )
    .unwrap();
    let token = g.battlefield.iter().find(|c| c.is_token).map(|c| c.id).expect("token");
    g.resolve_effect(
        &Effect::Destroy { what: Selector::Target(0) },
        &EffectContext { targets: vec![Target::Permanent(token)], ..ctx.clone() },
    )
    .unwrap();
    g.check_state_based_actions();
    assert!(!g.players[0].graveyard.iter().any(|c| c.id == token), "CR 111.7 — ceased to exist");
    let mut events = vec![];
    g.move_card_to(
        token,
        &ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
        &ctx,
        &mut events,
    );
    assert!(!g.battlefield.iter().any(|c| c.id == token), "CR 111.8 — it can't return");
}
