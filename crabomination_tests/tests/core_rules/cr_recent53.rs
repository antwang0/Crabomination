//! CR conformance for this run's engine work:
//! - CR 113.3 — any-player activated abilities (Damping Engine).
//! - CR 120.4a — excess damage redirected before the damage event.
//! - CR 208.3a — a P/T-modifying effect on a noncreature permanent is created
//!   anyway and applies once that permanent becomes a creature.
//! - CR 611.2c — "for as long as this remains tapped" continuous effects.

use crabomination::card::{CardType, CounterType};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::effect::{Duration, Effect, Selector, Value};
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target};
use crabomination::game::*;
use crabomination::TurnStep;

fn activate(g: &mut GameState, id: CardId, idx: usize) {
    g.perform_action(GameAction::ActivateAbility {
        card_id: id,
        ability_index: idx,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("activate");
    drain_stack(g);
}

// ── CR 113.3 — abilities ────────────────────────────────────────────────────

/// CR 113.3 — an ability flagged "any player may activate this" is usable by a
/// player who doesn't control the source; ordinary abilities are not.
#[test]
fn cr_113_3_any_player_may_activate_a_flagged_ability() {
    let mut g = two_player_game();
    let engine = g.add_card_to_battlefield(0, catalog::damping_engine());
    let jar = g.add_card_to_battlefield(0, catalog::memory_jar());
    g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.priority.player_with_priority = 1;
    assert!(
        g.perform_action(GameAction::ActivateAbility {
            card_id: jar,
            ability_index: 0,
            target: None,
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .is_err(),
        "an ordinary ability stays with its controller"
    );
    activate(&mut g, engine, 0);
    assert!(g.players[1].statics_ignored_this_turn.contains(&engine));
}

// ── CR 120.4a — excess damage ───────────────────────────────────────────────

/// CR 120.4a — the redirect happens before the damage is dealt, so the creature
/// is marked for exactly lethal and only the rest lands on its controller.
#[test]
fn cr_120_4a_excess_is_split_off_before_the_damage_event() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let life = g.players[1].life;
    let ctx = crabomination::game::effects::EffectContext::for_spell(
        0,
        Some(Target::Permanent(bear)),
        0,
        0,
    );
    g.resolve_effect(
        &Effect::DealDamageExcessToController {
            to: Selector::Target(0),
            amount: Value::Const(6),
        },
        &ctx,
    )
    .expect("resolve");
    assert_eq!(g.players[1].life, life - 4, "6 minus the lethal 2 spilled over");
    assert!(g.battlefield_find(bear).is_none());
}

/// CR 120.4a — a deathtouch source makes everything past 1 damage excess.
#[test]
fn cr_120_4a_deathtouch_makes_one_damage_lethal() {
    let mut g = two_player_game();
    let rats = g.add_card_to_battlefield(0, catalog::typhoid_rats());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let life = g.players[1].life;
    let ctx = crabomination::game::effects::EffectContext::for_ability(
        rats,
        0,
        Some(Target::Permanent(bear)),
    );
    g.resolve_effect(
        &Effect::DealDamageExcessToController {
            to: Selector::Target(0),
            amount: Value::Const(6),
        },
        &ctx,
    )
    .expect("resolve");
    assert_eq!(g.players[1].life, life - 5, "only 1 was needed on the body");
}

// ── CR 208.3a — P/T effects on noncreature permanents ───────────────────────

/// CR 208.3a — a +N/+N effect created against a noncreature permanent exists
/// and starts applying the moment that permanent becomes a creature.
#[test]
fn cr_208_3a_pump_on_a_noncreature_applies_once_it_animates() {
    let mut g = two_player_game();
    let jar = g.add_card_to_battlefield(0, catalog::memory_jar());
    let ctx = crabomination::game::effects::EffectContext::for_spell(
        0,
        Some(Target::Permanent(jar)),
        0,
        0,
    );
    g.resolve_effect(
        &Effect::PumpPT {
            what: Selector::Target(0),
            power: Value::Const(1),
            toughness: Value::Const(1),
            duration: Duration::EndOfTurn,
        },
        &ctx,
    )
    .expect("pump");
    assert!(!g.computed_permanent(jar).unwrap().card_types().contains(&CardType::Creature));
    g.resolve_effect(
        &Effect::BecomeCreature {
            what: Selector::Target(0),
            power: Value::Const(3),
            toughness: Value::Const(3),
            creature_types: vec![],
            keywords: vec![],
            duration: Duration::EndOfTurn,
        },
        &ctx,
    )
    .expect("animate");
    let cp = g.computed_permanent(jar).unwrap();
    assert_eq!((cp.power, cp.toughness), (4, 4), "the banked pump is live now");
}

// ── CR 611.2c — "for as long as" durations ──────────────────────────────────

/// CR 611.2c — the affected set is locked in when the effect starts, and the
/// effect ends when its stated condition (the source staying tapped) fails.
#[test]
fn cr_611_2c_while_source_tapped_locks_its_set_and_expires_on_untap() {
    let mut g = two_player_game();
    let weaponry = g.add_card_to_battlefield(0, catalog::thran_weaponry());
    let early = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add_colorless(2);
    activate(&mut g, weaponry, 0);
    let late = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    assert_eq!(g.computed_permanent(early).unwrap().power, 4);
    assert_eq!(g.computed_permanent(late).unwrap().power, 2, "joined after the lock-in");
    g.battlefield_find_mut(weaponry).unwrap().tapped = false;
    g.check_state_based_actions();
    assert_eq!(g.computed_permanent(early).unwrap().power, 2);
}

// ── CR 209 / 210 — loyalty and defense ──────────────────────────────────────

/// CR 209.1 / 306.5b — a planeswalker enters with loyalty counters equal to its
/// printed loyalty, and that number is its loyalty off the battlefield too.
#[test]
fn cr_209_1_planeswalker_enters_with_its_printed_loyalty() {
    use crabomination::card::CounterType;
    let mut g = two_player_game();
    let def = catalog::saheeli_sublime_artificer();
    let printed = def.base_loyalty;
    assert!(printed > 0, "the printed loyalty is the off-battlefield value");
    let pw = g.add_card_to_battlefield(0, def);
    assert_eq!(g.battlefield_find(pw).unwrap().counter_count(CounterType::Loyalty), printed);
}

/// CR 210.1 / 310.4b — a battle enters with defense counters equal to its
/// printed defense.
#[test]
fn cr_210_1_battle_enters_with_its_printed_defense() {
    use crabomination::card::CounterType;
    let mut g = two_player_game();
    let def = catalog::invasion_of_zendikar();
    let printed = def.defense;
    assert!(printed > 0);
    let battle = g.add_card_to_battlefield(0, def);
    assert_eq!(g.battlefield_find(battle).unwrap().counter_count(CounterType::Defense), printed);
}

// ── CR 308 — Kindred ────────────────────────────────────────────────────────

/// CR 308.1/308.2 — a kindred card has another card type and carries creature
/// types, so a type lord's anthem sees it as a Goblin without it being a
/// creature.
#[test]
fn cr_308_2_kindred_permanents_carry_creature_types_without_being_creatures() {
    use crabomination::card::{CardType, CreatureType};
    let mut g = two_player_game();
    let altar = g.add_card_to_battlefield(0, catalog::altar_of_the_goyf());
    let cp = g.computed_permanent(altar).unwrap();
    assert!(cp.card_types().contains(&CardType::Artifact));
    assert!(!cp.card_types().contains(&CardType::Creature), "a kindred artifact isn't a creature");
    assert!(cp.subtypes().creature_types.contains(&CreatureType::Lhurgoyf), "but it has the type");
}

// ── CR 603.2c / 603.3d / 603.4 / 603.6 — the graveyard walk ──────────────────
// `dispatch_triggers_for_events` walks graveyards for `FromYourGraveyard`
// triggers under the battlefield walk's rules: fan-out by kind, "once each
// turn" capping a batch at one fire and spending its slot only when the
// intervening if holds, "one or more" capping a batch without a turn cap.

/// CR 603.6 — a graveyard-resident "whenever an opponent gains life" fires
/// once per life-gain event in a batch, as it would from the battlefield.
#[test]
fn cr_603_6_a_graveyard_trigger_fans_out_per_matching_event() {
    let mut g = two_player_game();
    g.add_card_to_graveyard(0, catalog::punishing_fire());
    g.dispatch_triggers_for_events(&[
        GameEvent::LifeGained { player: 1, amount: 2 },
        GameEvent::LifeGained { player: 1, amount: 3 },
    ]);
    assert_eq!(g.stack.len(), 2, "one Punishing Fire trigger per life gain");
}

/// CR 603.3d — a graveyard-resident "once each turn" trigger fires once for a
/// batch of three draws and not again that turn.
#[test]
fn cr_603_3d_a_graveyard_once_per_turn_trigger_fires_once_a_batch_and_once_a_turn() {
    let mut g = two_player_game();
    g.add_card_to_graveyard(0, catalog::sneaky_snacker());
    let c = g.add_card_to_library(0, catalog::forest());
    g.players[0].cards_drawn_this_turn = 3;
    let draw = GameEvent::CardDrawn { player: 0, card_id: c };
    g.dispatch_triggers_for_events(&[draw.clone(), draw.clone(), draw.clone()]);
    assert_eq!(g.stack.len(), 1, "a draw-three batch mints one trigger");
    g.dispatch_triggers_for_events(&[draw]);
    assert_eq!(g.stack.len(), 1, "the slot is spent for the turn");
}

/// CR 603.4 — a failed intervening if does not spend the once-per-turn slot:
/// the second draw fails Sneaky Snacker's "your third card", the third fires.
#[test]
fn cr_603_4_a_failed_intervening_if_keeps_the_graveyard_once_per_turn_slot() {
    let mut g = two_player_game();
    g.add_card_to_graveyard(0, catalog::sneaky_snacker());
    let c = g.add_card_to_library(0, catalog::forest());
    let draw = GameEvent::CardDrawn { player: 0, card_id: c };
    g.players[0].cards_drawn_this_turn = 2;
    g.dispatch_triggers_for_events(std::slice::from_ref(&draw));
    assert!(g.stack.is_empty(), "the second draw is not the third");
    g.players[0].cards_drawn_this_turn = 3;
    g.dispatch_triggers_for_events(&[draw]);
    assert_eq!(g.stack.len(), 1, "the third draw fires");
}

/// CR 603.2c — "whenever one or more cards leave your graveyard" fires once
/// for a batch of two and again for the next batch (no per-turn cap).
#[test]
fn cr_603_2c_once_per_batch_fires_once_a_batch_and_again_next_batch() {
    let mut g = two_player_game();
    let hunter = g.add_card_to_battlefield(0, catalog::attuned_hunter());
    let c = g.add_card_to_library(0, catalog::forest());
    let left = GameEvent::CardLeftGraveyard { player: 0, card_id: c };
    g.dispatch_triggers_for_events(&[left.clone(), left.clone()]);
    assert_eq!(g.stack.len(), 1, "two cards leaving at once mint one trigger");
    drain_stack(&mut g);
    g.dispatch_triggers_for_events(&[left]);
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(hunter).unwrap().counter_count(CounterType::PlusOnePlusOne),
        2,
        "a later batch fires again"
    );
}

/// CR 603.2c — "whenever one or more creatures you control with flying deal
/// combat damage to a player" (Mu Yanling, Wind Rider) fires once for two
/// flyers connecting in one damage batch, not once per dealer.
#[test]
fn cr_603_2c_one_or_more_combat_damage_fires_once_a_damage_batch() {
    let mut g = two_player_game();
    let mu = g.add_card_to_battlefield(0, catalog::mu_yanling_wind_rider());
    g.battlefield_find_mut(mu).unwrap().counters.insert(CounterType::Loyalty, 5);
    let a = g.add_card_to_battlefield(0, catalog::suntail_hawk());
    let b = g.add_card_to_battlefield(0, catalog::storm_crow());
    g.clear_sickness(a);
    g.clear_sickness(b);
    for _ in 0..4 {
        g.add_card_to_library(0, catalog::forest());
    }
    let hand = g.players[0].hand.len();
    while g.step != TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).unwrap();
    }
    g.perform_action(GameAction::DeclareAttackers(vec![
        Attack { attacker: a, target: AttackTarget::Player(1) },
        Attack { attacker: b, target: AttackTarget::Player(1) },
    ]))
    .expect("attack");
    drain_stack(&mut g);
    while g.step != TurnStep::DeclareBlockers {
        g.perform_action(GameAction::PassPriority).unwrap();
    }
    g.perform_action(GameAction::DeclareBlockers(vec![])).expect("no block");
    while g.step != TurnStep::EndCombat {
        g.perform_action(GameAction::PassPriority).unwrap();
    }
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 18, "both flyers connected");
    assert_eq!(g.players[0].hand.len(), hand + 1, "one draw for the batch");
}

/// CR 603.2c — "whenever one or more other creatures you control die"
/// (Vengeful Townsfolk) fires once for a batch of two deaths and again for a
/// later death in the same turn: a batch cap, not a turn cap.
#[test]
fn cr_603_2c_one_or_more_deaths_fires_once_a_batch_and_again_later_in_the_turn() {
    let mut g = two_player_game();
    let vt = g.add_card_to_battlefield(0, catalog::vengeful_townsfolk());
    let a = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.dispatch_triggers_for_events(&[
        GameEvent::CreatureDied { card_id: a },
        GameEvent::CreatureDied { card_id: b },
    ]);
    assert_eq!(g.stack.len(), 1, "two deaths at once mint one trigger");
    drain_stack(&mut g);
    g.dispatch_triggers_for_events(&[GameEvent::CreatureDied { card_id: a }]);
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(vt).unwrap().counter_count(CounterType::PlusOnePlusOne),
        2,
        "a later death fires again"
    );
}

// ── CR 605.3a / 118.3 — paying a "you may pay" mid-resolution ─────────────

/// CR 605.3a — a "you may pay {R}" trigger resolving with an empty pool taps
/// the controller's Mountain for it (the echo path), instead of silently
/// failing the payment: Punishing Fire comes back to hand.
#[test]
fn cr_605_3a_a_you_may_pay_trigger_taps_a_land_when_the_pool_is_empty() {
    let mut g = two_player_game();
    let fire = g.add_card_to_graveyard(0, catalog::punishing_fire());
    let mountain = g.add_card_to_battlefield(0, catalog::mountain());
    assert_eq!(g.players[0].mana_pool.total(), 0);
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Bool(true)]));
    g.dispatch_triggers_for_events(&[GameEvent::LifeGained { player: 1, amount: 2 }]);
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == fire), "Punishing Fire returned");
    assert!(g.battlefield_find(mountain).unwrap().tapped, "the Mountain paid for it");
}

/// CR 605.3a — with no source to tap the payment fails and the body is
/// skipped, as before.
#[test]
fn cr_605_3a_an_unfundable_you_may_pay_runs_nothing() {
    let mut g = two_player_game();
    let fire = g.add_card_to_graveyard(0, catalog::punishing_fire());
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Bool(true)]));
    g.dispatch_triggers_for_events(&[GameEvent::LifeGained { player: 1, amount: 2 }]);
    drain_stack(&mut g);
    assert!(g.players[0].graveyard.iter().any(|c| c.id == fire), "stays in the graveyard");
}

// ── CR 603.6 — `EntersBattlefield` fans out over a simultaneous batch ────────
// A `CreateToken { count: 3 }` pushes three `PermanentEntered` events into one
// batch, and until `EventKind::EntersBattlefield` joined `event_kind_fans_out`
// a `YourControl`/`AnyPlayer` watcher fired ONCE for all of them: Soul Warden
// gained 1 life off Spectral Procession's three tokens. 332 catalog factories
// carry such a trigger and the whole suite passed either way, so these three
// are the gate.

/// CR 603.6 — the singular printed wording ("whenever another creature
/// enters") fires once per entering creature, end to end through the real
/// token-mint path.
#[test]
fn cr_603_6_an_etb_watcher_fires_once_per_entering_token() {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.add_card_to_battlefield(0, catalog::soul_warden());
    let life = g.players[0].life;
    let spell = g.add_card_to_hand(0, catalog::spectral_procession());
    g.players[0].mana_pool.add(crabomination::mana::Color::White, 10);
    g.players[0].mana_pool.add_colorless(10);
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast Spectral Procession");
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield.iter().filter(|c| c.definition.name == "Spirit").count(),
        3,
        "three tokens entered in one batch"
    );
    assert_eq!(g.players[0].life, life + 3, "one Soul Warden trigger per token, not one per batch");
}

/// CR 603.6 — "put **that many** +1/+1 counters" is spelled as one counter per
/// entering token, so the fan-out is what makes the total right. Pinning this
/// one with `once_per_batch` would put a single counter for a batch of three.
#[test]
fn cr_603_6_woodland_champion_counts_the_whole_token_batch() {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let champ = g.add_card_to_battlefield(0, catalog::woodland_champion());
    let spell = g.add_card_to_hand(0, catalog::spectral_procession());
    g.players[0].mana_pool.add(crabomination::mana::Color::White, 10);
    g.players[0].mana_pool.add_colorless(10);
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast Spectral Procession");
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(champ).unwrap().counter_count(CounterType::PlusOnePlusOne),
        3,
        "that many — one counter per token in the batch"
    );
}

/// CR 603.6 — the plural printed wording ("whenever **one or more** … enter")
/// pins itself with `once_per_batch` and mints one trigger for the batch.
/// Satoru draws one card however many uncast nontoken creatures arrive at once.
#[test]
fn cr_603_6_a_plural_etb_wording_mints_one_trigger_for_the_batch() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::satoru_the_infiltrator());
    let a = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let c = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.dispatch_triggers_for_events(&[
        GameEvent::PermanentEntered { card_id: a },
        GameEvent::PermanentEntered { card_id: b },
        GameEvent::PermanentEntered { card_id: c },
    ]);
    assert_eq!(g.stack.len(), 1, "one card for the batch, not one per creature");
}

/// The same batch against a singular watcher mints one trigger per match —
/// the contrast that makes the pin above mean something.
#[test]
fn cr_603_6_a_singular_etb_wording_mints_one_trigger_per_match() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::soul_warden());
    let a = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let c = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.dispatch_triggers_for_events(&[
        GameEvent::PermanentEntered { card_id: a },
        GameEvent::PermanentEntered { card_id: b },
        GameEvent::PermanentEntered { card_id: c },
    ]);
    assert_eq!(g.stack.len(), 3, "one Soul Warden trigger per entering creature");
}

// ── CR 603.6 — the rest of the fan-out list ─────────────────────────────────
// The `EntersBattlefield` pass above was the first row of a census of every
// `EventKind` the catalog triggers on under a scope a batch can carry several
// events for. Five more kinds were missing, each probed against a real batch
// first and each reading 1 where the printed card wants N. One gate per kind,
// plus the plural wordings that now have to pin themselves.

/// CR 603.6 — "whenever **a** card is put into an opponent's graveyard" is per
/// card, so a five-card mill is five +1/+1 counters, not one.
#[test]
fn cr_603_6_put_into_graveyard_fans_out_per_card() {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    let haunt = g.add_card_to_battlefield(0, catalog::the_haunt_of_hightower());
    for _ in 0..8 {
        g.add_card_to_library(1, catalog::grizzly_bears());
    }
    let scour = g.add_card_to_hand(0, catalog::tome_scour());
    g.players[0].mana_pool.add(crabomination::mana::Color::Blue, 5);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: scour,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Tome Scour");
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(haunt).unwrap().counter_count(CounterType::PlusOnePlusOne),
        5,
        "one counter per card that hit the graveyard"
    );
}

/// CR 603.6 — Moonshadow prints the plural ("whenever **one or more**
/// permanent cards are put into your graveyard"), so the same five-card mill
/// sheds exactly one -1/-1 counter.
#[test]
fn cr_603_6_a_plural_graveyard_wording_sheds_one_counter_for_the_batch() {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    let moon = g.add_card_to_battlefield(0, catalog::moonshadow());
    // `add_card_to_battlefield` skips the entry replacement, so stamp the six
    // -1/-1 counters the card enters with by hand.
    g.battlefield_find_mut(moon).unwrap().add_counters(CounterType::MinusOneMinusOne, 6);
    let before = g.battlefield_find(moon).unwrap().counter_count(CounterType::MinusOneMinusOne);
    assert_eq!(before, 6);
    for _ in 0..8 {
        g.add_card_to_library(0, catalog::grizzly_bears());
    }
    let scour = g.add_card_to_hand(0, catalog::tome_scour());
    g.players[0].mana_pool.add(crabomination::mana::Color::Blue, 5);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: scour,
        target: Some(Target::Player(0)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Tome Scour");
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(moon).unwrap().counter_count(CounterType::MinusOneMinusOne),
        before - 1,
        "one counter for the batch, not one per card"
    );
}

/// CR 603.6 — "whenever **a** land card is put into your graveyard" is per
/// land: Slogurk takes five counters off a five-land mill.
#[test]
fn cr_603_6_land_put_into_graveyard_fans_out_per_land() {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    let slog = g.add_card_to_battlefield(0, catalog::slogurk_the_overslime());
    for _ in 0..8 {
        g.add_card_to_library(0, catalog::forest());
    }
    let scour = g.add_card_to_hand(0, catalog::tome_scour());
    g.players[0].mana_pool.add(crabomination::mana::Color::Blue, 5);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: scour,
        target: Some(Target::Player(0)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Tome Scour");
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(slog).unwrap().counter_count(CounterType::PlusOnePlusOne),
        5,
        "one counter per milled land"
    );
}

/// CR 603.2c — landfall is "whenever **a** land you control enters": two lands
/// entering in one batch (a blink, Entish Restoration's search) are two
/// triggers. `LandPlayed` read a batch as one until it joined
/// `event_kind_fans_out`.
#[test]
fn cr_603_2c_landfall_fans_out_per_land() {
    let mut g = two_player_game();
    let kudzu = g.add_card_to_battlefield(0, catalog::vinelasher_kudzu());
    let a = g.add_card_to_battlefield(0, catalog::forest());
    let b = g.add_card_to_battlefield(0, catalog::forest());
    g.dispatch_triggers_for_events(&[
        GameEvent::LandPlayed { player: 0, card_id: a, played: false },
        GameEvent::LandPlayed { player: 0, card_id: b, played: false },
    ]);
    let from_kudzu = g
        .stack
        .iter()
        .filter(|item| matches!(item, StackItem::Trigger { source, .. } if *source == kudzu))
        .count();
    assert_eq!(from_kudzu, 2, "one landfall trigger per land");
}

/// CR 603.6 — "whenever you create **a** token" is per token: three Spirits
/// from one Spectral Procession drain for three.
#[test]
fn cr_603_6_token_created_fans_out_per_token() {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.add_card_to_battlefield(0, catalog::mirkwood_bats());
    let life = g.players[1].life;
    let spell = g.add_card_to_hand(0, catalog::spectral_procession());
    g.players[0].mana_pool.add(crabomination::mana::Color::White, 10);
    g.players[0].mana_pool.add_colorless(10);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Spectral Procession");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life - 3, "one drain per token, not one per batch");
}

/// CR 603.6 — "whenever **a** creature you control explores" is per explore.
#[test]
fn cr_603_6_explored_fans_out_per_explore() {
    let mut g = two_player_game();
    let walker = g.add_card_to_battlefield(0, catalog::wildgrowth_walker());
    let a = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.dispatch_triggers_for_events(&[
        GameEvent::Explored { card_id: a, controller: 0, explored_land: false },
        GameEvent::Explored { card_id: b, controller: 0, explored_land: false },
    ]);
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(walker).unwrap().counter_count(CounterType::PlusOnePlusOne),
        2,
        "one counter per exploring creature"
    );
}

/// CR 603.6 — "whenever **a** creature deals damage to you, put a gold counter
/// on **it**" is per creature, and two attackers really do arrive as two
/// `DamageDealt` events in one batch. Pinned, only the first attacker would be
/// walled.
#[test]
fn cr_603_6_player_damaged_fans_out_per_source() {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.add_card_to_battlefield(1, catalog::aurification());
    let a = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    for id in [a, b] {
        g.clear_sickness(id);
    }
    g.step = TurnStep::DeclareAttackers;
    g.declare_attackers(vec![
        Attack { attacker: a, target: AttackTarget::Player(1) },
        Attack { attacker: b, target: AttackTarget::Player(1) },
    ])
    .expect("attack");
    while g.step != TurnStep::CombatDamage {
        g.perform_action(GameAction::PassPriority).expect("pass priority");
    }
    drain_stack(&mut g);
    let gold: u32 = [a, b]
        .iter()
        .filter_map(|id| g.battlefield_find(*id))
        .map(|c| c.counter_count(CounterType::Gold))
        .sum();
    assert_eq!(gold, 2, "a gold counter on each attacker, not just the first");
}

/// CR 603.6 — "whenever **a** creature is exiled from the battlefield" is per
/// creature.
#[test]
fn cr_603_6_card_exiled_fans_out_per_card() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::soulherder());
    let a = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.dispatch_triggers_for_events(&[
        GameEvent::PermanentExiled { card_id: a },
        GameEvent::PermanentExiled { card_id: b },
    ]);
    assert_eq!(g.stack.len(), 2, "one Soulherder trigger per exiled creature");
}

/// The graveyard walk is the battlefield walk's twin and has to carry the same
/// rules. `EventKind::PutIntoGraveyard` is a `is_graveyard_self_source_kind`,
/// so a `SelfSource` "when this is put into a graveyard from anywhere" fires
/// out of the graveyard the card just landed in — and a mill puts BOTH a
/// `CardPutIntoGraveyard` and a `CardMilled` for it in the batch. The
/// subject dedupe that the battlefield walk got has to be there too, or Ichor
/// Wellspring draws two cards for one mill.
#[test]
fn cr_603_6_a_milled_self_source_graveyard_trigger_fires_once() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::ichor_wellspring());
    for _ in 0..4 {
        g.add_card_to_library(0, catalog::forest()); // something to draw
    }
    let before = g.players[0].hand.len();
    let evs = g
        .resolve_effect(
            &crabomination::effect::Effect::Mill {
                who: Selector::Player(crabomination::effect::PlayerRef::You),
                amount: Value::ONE,
            },
            &crabomination::game::effects::EffectContext::for_spell(0, None, 0, 0),
        )
        .expect("mill one");
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(
        g.players[0].hand.len(),
        before + 1,
        "one card reaching the graveyard is one trigger, whichever records the batch carries",
    );
}
