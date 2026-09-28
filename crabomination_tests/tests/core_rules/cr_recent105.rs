//! CR 614 — "If it would leave the battlefield, exile it instead of putting
//! it anywhere else" (`Effect::ExileIfLeavesBattlefield`): a replacement
//! bound to one object, covering every exit (death, bounce), spent when that
//! object leaves (CR 400.7). It used to be a finality counter (Geth), an
//! end-of-turn death redirect (Gruesome Encore), or nothing (Whip of Erebos,
//! Llanowar Greenwidow), so a bounce kept the creature.

use crabomination::card::{CardDefinition, CardId};
use crabomination::catalog;
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

/// Geth reanimates a Bears under the replacement; returns (game, bears).
fn geth_returns_bears() -> (GameState, CardId) {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let geth = g.add_card_to_battlefield(0, catalog::geth_thane_of_contracts());
    g.battlefield_find_mut(geth).unwrap().summoning_sick = false;
    let bears = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Black, 3);
    g.perform_action(GameAction::ActivateAbility {
        card_id: geth,
        ability_index: 0,
        target: Some(Target::Permanent(bears)),
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("Geth reanimates");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bears).is_some());
    (g, bears)
}

fn cast(g: &mut GameState, id: CardId, target: CardId) {
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(target)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(g);
}

fn in_exile(g: &GameState, id: CardId) -> bool {
    g.exile.iter().any(|c| c.id == id)
}

#[test]
fn cr_614_a_reanimated_creature_that_dies_is_exiled() {
    let (mut g, bears) = geth_returns_bears();
    let murder = g.add_card_to_hand(0, catalog::murder());
    g.players[0].mana_pool.add(Color::Black, 3);
    cast(&mut g, murder, bears);
    assert!(in_exile(&g, bears));
    assert!(g.players[0].graveyard.iter().all(|c| c.id != bears));
}

#[test]
fn cr_614_a_bounced_one_is_exiled_too_and_the_replacement_is_spent() {
    let (mut g, bears) = geth_returns_bears();
    let unsummon = g.add_card_to_hand(0, catalog::unsummon());
    g.players[0].mana_pool.add(Color::Blue, 1);
    cast(&mut g, unsummon, bears);
    assert!(in_exile(&g, bears), "not back in hand");
    assert!(g.players[0].hand.iter().all(|c| c.id != bears));
    // CR 400.7 — whatever comes back later is a new object.
    assert!(g.replacement_effects.is_empty(), "the object-bound replacement lapsed");
}

/// CR 614.13 — stacked token doublers multiply, and every mint loop is capped
/// at the simulator's board bound: forty Adrix and Nevs doubling a Cadric
/// copy, a populate and a token copy used to ask for 2^32 mints (an
/// eight-seat pod hung inside one bot probe).
#[test]
fn cr_614_13_forty_token_doublers_mint_a_bounded_batch() {
    use crabomination::card::Value;
    use crabomination::effect::{Effect, PlayerRef, Selector};
    let mut g = two_player_game();
    for _ in 0..40 {
        g.add_card_to_battlefield(0, catalog::adrix_and_nev_twincasters());
    }
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let mut ctx = crabomination::game::effects::EffectContext::for_spell(0, None, 0, 0);
    ctx.targets = vec![Target::Permanent(bear)];
    let before = g.battlefield.len();
    g.resolve_effect(
        &Effect::CreateTokenCopiesHasteSac { who: PlayerRef::You, count: Value::ONE, source: Selector::Target(0), exile: false },
        &ctx,
    )
    .expect("copies");
    let made = g.battlefield.len() - before;
    // The batch is sized to the bound (1,025) and the mint stops once the
    // board passes it (`a_token_chain_stops_at_the_board_bound`), so the
    // board ends one past the bound with the 41 permanents already out.
    assert!(made <= 1025, "doubled to the board bound, not 2^40: {made}");
    assert_eq!(g.battlefield.len(), crabomination::recommend::MAX_BATTLEFIELD + 1, "made {made}");
}

/// CR 614.13 — a token doubler doubles tokens created tapped and attacking
/// too (Mobilize, Myriad-style riders): one Adrix and Nev turns one attacking
/// token into two. The attacking-token mint used to skip the doublers.
#[test]
fn cr_614_13_tokens_created_attacking_are_doubled() {
    use crabomination::card::Value;
    use crabomination::effect::{AttackingTokenCleanup, Effect, PlayerRef};
    use crabomination::game::types::{Attack, AttackTarget};
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::adrix_and_nev_twincasters());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.step = TurnStep::DeclareAttackers;
    g.attacking.push(Attack { attacker: bear, target: AttackTarget::Player(1) });
    let mut ctx = crabomination::game::effects::EffectContext::for_spell(0, None, 0, 0);
    ctx.source = Some(bear);
    g.resolve_effect(
        &Effect::CreateTokenAttacking {
            who: PlayerRef::You,
            count: Value::ONE,
            definition: std::sync::Arc::new(crabomination_base::tokens::spirit_token()),
            cleanup: AttackingTokenCleanup::default(),
            defender: None,
        },
        &ctx,
    )
    .expect("tokens");
    let attackers = g.attacking.iter().filter(|a| g.battlefield_find(a.attacker).is_some_and(|c| c.is_token)).count();
    assert_eq!(attackers, 2, "doubled, both attacking");
}

/// A token-mint chain is bounded by the simulator's board bound too, not just
/// one batch: a board already past `recommend::MAX_BATTLEFIELD` (where the
/// game ends as a board cap regardless) mints nothing more. Two Redoubled
/// Stormsingers under two Harmonic Prodigies copy every token that entered
/// this turn, three times a trigger, and a bot's sim of one six-seat combat
/// never returned — each mint walked the whole board.
#[test]
fn a_token_chain_stops_at_the_board_bound() {
    use crabomination::card::Value;
    use crabomination::effect::{Effect, PlayerRef};
    let mut g = two_player_game();
    let ctx = crabomination::game::effects::EffectContext::for_spell(0, None, 0, 0);
    let batch = Effect::CreateToken {
        who: PlayerRef::You,
        count: Value::Const(600),
        definition: std::sync::Arc::new(crabomination_base::tokens::spirit_token()),
    };
    for _ in 0..4 {
        g.resolve_effect(&batch, &ctx).expect("tokens");
    }
    let bound = crabomination::recommend::MAX_BATTLEFIELD;
    assert!(
        g.battlefield.len() > bound && g.battlefield.len() <= bound + 600,
        "the chain stops at the first batch past the bound: {}",
        g.battlefield.len(),
    );
}

/// A spell-copy chain is bounded by the simulator's stack bound
/// (`recommend::MAX_STACK`): a Venser, Fervent Forger copying a Replication
/// Technique that copies Venser grew a pod's stack to 1,692 items until one
/// action never returned. Ten thousand copies asked for stop at the bound.
#[test]
fn a_spell_copy_chain_stops_at_the_stack_bound() {
    use crabomination::card::Value;
    use crabomination::effect::{Effect, Selector};
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)), additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("bolt");
    let mut ctx = crabomination::game::effects::EffectContext::for_spell(0, None, 0, 0);
    ctx.targets = vec![Target::Permanent(bolt)];
    g.resolve_effect(&Effect::CopySpell { what: Selector::Target(0), count: Value::Const(10_000) }, &ctx)
        .expect("copies");
    assert!(g.stack.len() <= crabomination::recommend::MAX_STACK + 1, "stack {}", g.stack.len());
    assert!(g.stack.len() > 100, "the copies up to the bound were made");
}

// ── CR 505.1 — "your next main phase" is the first OR the second ────────────

/// Pass priority until `step` begins (the stack is drained on the way).
fn pass_to_step(g: &mut GameState, step: TurnStep) {
    for _ in 0..200 {
        if g.step == step {
            return;
        }
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    panic!("never reached {step:?}");
}

/// Seat 1 Bolts seat 0 during seat 0's first main phase; seat 0 answers with
/// `counter`, paying `blue` of its cost in {U} and the rest in {G}.
fn counter_a_bolt_in_first_main(g: &mut GameState, counter: CardDefinition, blue: u32) {
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    let md = g.add_card_to_hand(0, counter);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.players[1].mana_pool.add(Color::Red, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Player(0)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Bolt");
    g.players[0].mana_pool.add(Color::Blue, blue);
    g.players[0].mana_pool.add(Color::Green, 4 - blue.min(4));
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: md,
        target: Some(Target::Permanent(bolt)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("counter");
    drain_stack(g);
    assert!(g.players[1].graveyard.iter().any(|c| c.id == bolt), "Bolt countered");
    g.players[0].mana_pool.empty();
}

/// CR 505.1 — Mana Drain: "At the beginning of your next main phase, add {C}
/// equal to that spell's mana value." Countering in your own first main
/// phase banks for the SECOND main phase this turn; the rider was missing
/// entirely, and the kind it would have used fires on a first main only.
#[test]
fn cr_505_1_mana_drain_pays_out_at_this_turns_second_main_phase() {
    let mut g = two_player_game();
    counter_a_bolt_in_first_main(&mut g, catalog::mana_drain(), 2);
    let turn = g.turn_number;
    pass_to_step(&mut g, TurnStep::PostCombatMain);
    drain_stack(&mut g);
    assert_eq!(g.turn_number, turn, "still this turn");
    assert_eq!(g.players[0].mana_pool.colorless_amount(), 1, "Bolt's mana value in {{C}}");
}

/// CR 505.1a — Plasm Capture says "your next FIRST main phase": a counter in
/// your first main phase waits a whole turn cycle, it doesn't pay out in the
/// second main phase.
#[test]
fn cr_505_1a_plasm_capture_waits_for_a_first_main_phase() {
    let mut g = two_player_game();
    counter_a_bolt_in_first_main(&mut g, catalog::plasm_capture(), 2);
    pass_to_step(&mut g, TurnStep::PostCombatMain);
    drain_stack(&mut g);
    assert_eq!(g.players[0].mana_pool.total(), 0, "nothing in the second main phase");
    assert!(g.delayed_triggers.iter().any(|dt| dt.controller == 0), "still waiting");
}

/// Vivien's Stampede — "At the beginning of the next main phase this turn,
/// draw a card for each player who was dealt combat damage this turn": the
/// draw comes in the second main phase (it came at end of combat), and cast
/// in the second main phase there is no main phase left, so the trigger
/// lapses at cleanup (CR 603.7b, a stated "this turn" duration).
#[test]
fn cr_603_7b_viviens_stampede_draws_at_the_next_main_phase_this_turn_only() {
    for (cast_step, draws) in [(TurnStep::PreCombatMain, 1), (TurnStep::PostCombatMain, 0)] {
        let mut g = two_player_game();
        for _ in 0..3 {
            g.add_card_to_library(0, catalog::grizzly_bears());
        }
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.clear_sickness(bear);
        let stampede = g.add_card_to_hand(0, catalog::viviens_stampede());
        g.active_player_idx = 0;
        g.step = cast_step;
        g.priority.player_with_priority = 0;
        g.players[0].mana_pool.add(Color::Green, 6);
        crabomination::game::cast(&mut g, stampede);
        let hand = g.players[0].hand.len();
        let turn = g.turn_number;
        if cast_step == TurnStep::PreCombatMain {
            pass_to_step(&mut g, TurnStep::DeclareAttackers);
            g.perform_action(GameAction::DeclareAttackers(vec![Attack {
                attacker: bear,
                target: AttackTarget::Player(1),
            }]))
            .expect("attack");
            pass_to_step(&mut g, TurnStep::PostCombatMain);
            drain_stack(&mut g);
            assert!(g.players[1].life < 20, "the Bear connected");
        } else {
            while g.turn_number == turn {
                g.perform_action(GameAction::PassPriority).expect("pass");
            }
            assert!(g.delayed_triggers.iter().all(|dt| dt.source != stampede), "lapsed at cleanup");
        }
        assert_eq!(g.players[0].hand.len() - hand, draws, "cast in {cast_step:?}");
    }
}
