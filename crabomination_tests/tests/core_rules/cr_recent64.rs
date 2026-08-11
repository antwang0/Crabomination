//! CR conformance for this run's engine work:
//! - CR 705.1 — "flip a coin until you lose a flip".
//! - CR 702.15 — Landwalk keys on the defending player's lands.
//! - CR 702.34a / 601.2b — flashback-only additional costs.
//! - CR 611.2b — an indefinite continuous effect outlives its source.

use crabomination::card::{CounterType, Keyword, LandType};
use crabomination::catalog;
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target};
use crabomination::game::*;
use crabomination::mana::Color;

fn main_phase() -> GameState {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g
}

fn mana(g: &mut GameState, seat: usize) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[seat].mana_pool.add(c, 20);
    }
    g.players[seat].mana_pool.add_colorless(20);
}

/// CR 705.1 — the flip-until-loss loop pays out once per won flip, and the
/// counters it left match the wins the log recorded.
#[test]
fn cr_705_1_flip_until_loss_pays_per_win() {
    let mut g = main_phase();
    let cat = g.add_card_to_battlefield(0, catalog::crazed_firecat());
    g.fire_self_etb_triggers(cat, 0);
    let events = drain_stack(&mut g);
    let wins = events
        .iter()
        .filter(|e| matches!(e, GameEvent::CoinFlipWon { player: 0 }))
        .count() as u32;
    let losses = events
        .iter()
        .filter(|e| matches!(e, GameEvent::CoinFlipLost { player: 0 }))
        .count();
    assert_eq!(losses, 1, "the loop stops on the first loss");
    assert_eq!(
        g.battlefield_find(cat).unwrap().counter_count(CounterType::PlusOnePlusOne),
        wins,
        "one +1/+1 counter per won flip"
    );
}

/// CR 702.15b — a swampwalker is unblockable only while the *defending* player
/// controls a Swamp.
#[test]
fn cr_702_15b_landwalk_keys_on_the_defender() {
    let mut g = main_phase();
    let snake = g.add_card_to_battlefield(0, catalog::krosan_constrictor());
    let blocker = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(snake);
    // Attacker's own Swamp doesn't grant the evasion.
    g.add_card_to_battlefield(0, catalog::swamp());
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: snake,
        target: AttackTarget::Player(1),
    }]))
    .expect("attack");
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = 1;
    assert!(
        g.clone()
            .perform_action(GameAction::DeclareBlockers(vec![(blocker, snake)]))
            .is_ok(),
        "no Swamp on the defending side, so the bear can block"
    );
    g.add_card_to_battlefield(1, catalog::swamp());
    assert!(
        g.perform_action(GameAction::DeclareBlockers(vec![(blocker, snake)])).is_err(),
        "the defender's Swamp turns on swampwalk"
    );
    assert!(
        g.battlefield_find(snake)
            .unwrap()
            .definition
            .keywords
            .contains(&Keyword::Landwalk(LandType::Swamp))
    );
}

/// CR 702.34a — a flashback cast pays the card's flashback-only additional
/// cost on top of the flashback mana cost.
#[test]
fn cr_702_34a_flashback_additional_cost_is_paid() {
    let mut g = main_phase();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let spell = g.add_card_to_graveyard(0, catalog::crippling_fatigue());
    mana(&mut g, 0);
    let life = g.players[0].life;
    g.perform_action(GameAction::CastFlashback {
        card_id: spell,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("flash back");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life - 3, "the pay-3-life rider was paid");
    assert!(g.exile.iter().any(|c| c.id == spell), "and the card was exiled (702.34d)");
}

/// CR 611.2b — a continuous effect with no duration doesn't end when the
/// object that created it leaves the battlefield.
#[test]
fn cr_611_2b_indefinite_effect_outlives_its_source() {
    use crabomination::card::CreatureType;
    let mut g = main_phase();
    let corpse = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let chainer = g.add_card_to_battlefield(0, catalog::chainer_dementia_master());
    mana(&mut g, 0);
    g.perform_action(GameAction::ActivateAbility {
        card_id: chainer,
        ability_index: 0,
        target: Some(Target::Permanent(corpse)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("reanimate");
    drain_stack(&mut g);
    assert!(
        g.computed_permanent(corpse)
            .unwrap()
            .subtypes
            .creature_types
            .contains(&CreatureType::Nightmare),
        "reanimated as a Nightmare"
    );
    // The stamp is sourced from Chainer but has no duration, so removing the
    // *anthem* source must not remove it. Verified through Chainer's own exile
    // trigger, which only sees Nightmares.
    let mut events = Vec::new();
    g.destroy_permanent(chainer, false, &mut events);
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == corpse), "the Nightmare was exiled");
}

/// CR 705.1 — the flip is drawn from the `GameState`'s own seeded stream, so
/// a fixed seed reflips identically in the next process. It used to read
/// `rand::random()` (the thread RNG) inside `AutoDecider`, and Mana Crypt's
/// upkeep flip is in the cube pool: every cube/`all` bench number was
/// therefore unreproducible on a fixed seed.
#[test]
fn cr_705_1_flips_come_off_the_seeded_game_stream() {
    let wins = |seed: u64| {
        let mut g = main_phase();
        g.rng.reseed(seed);
        let cat = g.add_card_to_battlefield(0, catalog::crazed_firecat());
        g.fire_self_etb_triggers(cat, 0);
        drain_stack(&mut g)
            .iter()
            .filter(|e| matches!(e, GameEvent::CoinFlipWon { .. }))
            .count()
    };
    assert_eq!(wins(7), wins(7), "same seed flipped a different number of heads");
    assert!(
        (0..32).any(|s| wins(s) != wins(0)),
        "every seed flipped the same coins — the flip isn't reading the stream"
    );
}
