//! Commander: the Entropic Uprising precon (C16, Yidris,
//! `decks::cmdr_yidris`) and the primitives it needed.

use crabomination::card::{SelectionRequirement as R, CardId, Keyword, CardDefinition, CardType, CounterType, EventKind, EventScope, EventSpec, TriggeredAbility, Value};
use crabomination::catalog;
use crabomination::effect::{Effect, PlayerRef, Selector};
use crabomination::game::effects::EffectContext;

fn run(g: &mut GameState, seat: usize, e: &Effect) {
    let ctx = EffectContext::for_spell(seat, None, 0, 0);
    g.resolve_effect(e, &ctx).expect("resolve");
    drain_stack(g);
}
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target};
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::mana::Color;
use crabomination::game::types::TurnStep;
use crabomination::game::*;

fn pod(n: usize) -> GameState {
    let mut g = multi_player_game(n);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g
}

/// CR 800.4a — "whenever a player loses the game": each other player's
/// permanent triggers once per departing player.
#[test]
fn cr_800_4a_a_permanent_sees_each_player_leave() {
    let watcher = || CardDefinition {
        name: "Test Watcher",
        card_types: vec![CardType::Creature],
        power: 1,
        toughness: 1,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PlayerLeftGame, EventScope::SelfSource),
            effect: Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::Const(5) },
        }],
        ..Default::default()
    };
    let mut g = pod(4);
    let mine = g.add_card_to_battlefield(0, watcher());
    let theirs = g.add_card_to_battlefield(3, watcher());
    g.players[1].life = 0;
    g.players[2].life = 0;
    g.check_state_based_actions();
    drain_stack(&mut g);
    for id in [mine, theirs] {
        assert_eq!(g.battlefield_find(id).unwrap().counter_count(CounterType::PlusOnePlusOne), 10, "two players left");
    }
}

/// CR 723.1 — each of two players controls the other's next turn; the swap
/// takes effect on the turn itself, and the caster controls nobody.
#[test]
fn cr_723_1_two_players_control_each_others_next_turn() {
    let mut g = pod(3);
    let mut ctx = EffectContext::for_spell(0, Some(Target::Player(1)), 0, 0);
    ctx.targets = vec![Target::Player(1), Target::Player(2)];
    g.resolve_effect(
        &Effect::PlayersControlEachOthersNextTurn { first: PlayerRef::Target(0), second: PlayerRef::Target(1) },
        &ctx,
    )
    .expect("resolve");
    g.apply_pending_player_control(1);
    assert_eq!(g.controlled_by.get(1).copied().flatten(), Some(2), "seat 2 runs seat 1's turn");
    g.apply_pending_player_control(2);
    assert_eq!(g.controlled_by.get(2).copied().flatten(), Some(1), "seat 1 runs seat 2's turn");
    g.apply_pending_player_control(0);
    assert!(g.controlled_by.is_empty(), "the caster's own turn is theirs");
}

/// CR 701.34 — every creature card in the target's graveyard is manifested
/// under the caster: face-down 2/2s; the non-creature card stays.
#[test]
fn cr_701_34_manifest_a_graveyard_under_your_control() {
    let mut g = pod(2);
    g.add_card_to_graveyard(1, catalog::serra_angel());
    g.add_card_to_graveyard(1, catalog::grizzly_bears());
    let bolt = g.add_card_to_graveyard(1, catalog::lightning_bolt());
    let mut ctx = EffectContext::for_spell(0, Some(Target::Player(1)), 0, 0);
    ctx.targets = vec![Target::Player(1)];
    g.resolve_effect(&Effect::ManifestFromGraveyard { who: PlayerRef::Target(0), filter: R::Creature }, &ctx)
        .expect("resolve");
    let manifested: Vec<_> = g.battlefield.iter().filter(|c| c.controller == 0 && c.face_down).map(|c| c.id).collect();
    assert_eq!(manifested.len(), 2);
    for id in manifested {
        let cp = g.computed_permanent(id).unwrap();
        assert_eq!((cp.power, cp.toughness), (2, 2));
        assert_eq!(g.battlefield_find(id).unwrap().owner, 1, "still the opponent's card");
    }
    assert!(g.players[1].graveyard.iter().any(|c| c.id == bolt));
}

// ── The cards ──

fn flood(g: &mut GameState, seat: usize) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[seat].mana_pool.add(c, 20);
    }
    g.players[seat].mana_pool.add_colorless(20);
}

fn cast_with(g: &mut GameState, seat: usize, id: CardId, target: Option<Target>, more: Vec<Target>) -> Result<(), String> {
    flood(g, seat);
    g.priority.player_with_priority = seat;
    g.perform_action(GameAction::CastSpell { card_id: id, target, additional_targets: more, mode: None, x_value: None })
        .map(|_| ())
        .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn cast(g: &mut GameState, seat: usize, id: CardId, target: Option<Target>) -> Result<(), String> {
    cast_with(g, seat, id, target, vec![])
}

fn activate(g: &mut GameState, seat: usize, id: CardId, index: usize, target: Option<Target>) -> Result<(), String> {
    flood(g, seat);
    g.priority.player_with_priority = seat;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id,
        ability_index: index,
        target,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .map(|_| ())
    .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn named(g: &GameState, seat: usize, name: &str) -> Vec<CardId> {
    g.battlefield.iter().filter(|c| c.controller == seat && c.definition.name == name).map(|c| c.id).collect()
}

fn library(g: &mut GameState, seat: usize, n: usize) {
    for _ in 0..n {
        g.add_card_to_library(seat, catalog::island());
    }
}

fn connect(g: &mut GameState, a: CardId) {
    g.clear_sickness(a);
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: a, target: AttackTarget::Player(1) }]))
        .expect("attack");
    drain_stack(g);
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::DeclareBlockers(vec![])).expect("no blocks");
    while g.step != TurnStep::EndCombat {
        let _ = g.advance_step(Vec::new());
        drain_stack(g);
    }
}

fn upkeep(g: &mut GameState) {
    g.step = TurnStep::Upkeep;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(g);
}

/// After Yidris connects, a spell cast from hand cascades.
#[test]
fn yidris_gives_hand_spells_cascade() {
    let mut g = pod(2);
    let y = g.add_card_to_battlefield(0, catalog::yidris_maelstrom_wielder());
    connect(&mut g, y);
    g.add_card_to_library(0, catalog::grizzly_bears());
    g.step = TurnStep::PostCombatMain;
    let angel = g.add_card_to_hand(0, catalog::serra_angel());
    cast(&mut g, 0, angel, None).expect("cast");
    assert_eq!(named(&g, 0, "Grizzly Bears").len(), 1, "cascaded into the Bears");
    // A spell cast from the graveyard doesn't cascade.
    g.add_card_to_library(0, catalog::grizzly_bears());
    let bolt = g.add_card_to_graveyard(0, catalog::lightning_bolt());
    run(&mut g, 0, &Effect::GrantMayPlay {
        what: Selector::EachMatching {
            zone: crabomination::effect::ZoneRef::Graveyard(PlayerRef::You),
            filter: R::HasCardType(CardType::Instant),
        },
        duration: crabomination::card::MayPlayDuration::EndOfThisTurn,
        to_owner: false,
        exile_after: false,
        pay_own_cost: true,
        any_color: false,
    });
    flood(&mut g, 0);
    g.perform_action(GameAction::CastFromZoneWithoutPaying {
        card_id: bolt,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast from the graveyard");
    drain_stack(&mut g);
    assert_eq!(named(&g, 0, "Grizzly Bears").len(), 1, "no cascade from the graveyard");
}

/// Its size is your hand.
#[test]
fn aeon_chronicler_counts_the_hand() {
    let mut g = pod(2);
    let a = g.add_card_to_battlefield(0, catalog::aeon_chronicler());
    for _ in 0..3 {
        g.add_card_to_hand(0, catalog::island());
    }
    assert_eq!(g.computed_permanent(a).unwrap().power, 3);
}

/// Each upkeep everyone bleeds and it grows by one per player; a player
/// leaving adds five.
#[test]
fn blood_tyrant_feeds() {
    let mut g = pod(3);
    let t = g.add_card_to_battlefield(0, catalog::blood_tyrant());
    let life: Vec<i32> = g.players.iter().map(|p| p.life).collect();
    upkeep(&mut g);
    assert!(g.players.iter().zip(&life).all(|(p, l)| p.life == l - 1));
    assert_eq!(g.battlefield_find(t).unwrap().counter_count(CounterType::PlusOnePlusOne), 3);
    g.players[2].life = 0;
    g.check_state_based_actions();
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(t).unwrap().counter_count(CounterType::PlusOnePlusOne), 8);
}

/// Two target players, each to control the other's next turn.
#[test]
fn cruel_entertainment_swaps_two_players() {
    let mut g = pod(3);
    let s = g.add_card_to_hand(0, catalog::cruel_entertainment());
    cast_with(&mut g, 0, s, Some(Target::Player(1)), vec![Target::Player(2)]).expect("cast");
    g.apply_pending_player_control(1);
    assert_eq!(g.controlled_by.get(1).copied().flatten(), Some(2));
}

/// Every nonland permanent to its owner's hand.
#[test]
fn devastation_tide_bounces_everything() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_battlefield(1, catalog::serra_angel());
    let land = g.add_card_to_battlefield(1, catalog::island());
    let s = g.add_card_to_hand(0, catalog::devastation_tide());
    cast(&mut g, 0, s, None).expect("cast");
    assert!(g.players[1].hand.iter().any(|c| c.definition.name == "Serra Angel"));
    assert!(g.players[0].hand.iter().any(|c| c.definition.name == "Grizzly Bears"));
    assert!(g.battlefield_find(land).is_some());
}

/// Entering, it steals the enchanted permanent for the turn, untapped and
/// hasty.
#[test]
fn frenzied_fugue_borrows_a_permanent() {
    let mut g = pod(2);
    let angel = g.add_card_to_battlefield(1, catalog::serra_angel());
    g.battlefield_find_mut(angel).unwrap().tapped = true;
    let f = g.add_card_to_hand(0, catalog::frenzied_fugue());
    cast(&mut g, 0, f, Some(Target::Permanent(angel))).expect("cast");
    let a = g.battlefield_find(angel).unwrap();
    assert_eq!(a.controller, 0);
    assert!(!a.tapped);
    assert!(g.computed_permanent(angel).unwrap().keywords().contains(&Keyword::Haste));
}

/// Manifests the target's dead creatures for you.
#[test]
fn ghastly_conscription_raises_face_down() {
    let mut g = pod(2);
    g.add_card_to_graveyard(1, catalog::serra_angel());
    let s = g.add_card_to_hand(0, catalog::ghastly_conscription());
    cast(&mut g, 0, s, Some(Target::Player(1))).expect("cast");
    assert_eq!(g.battlefield.iter().filter(|c| c.controller == 0 && c.face_down).count(), 1);
}

/// At an opponent's end step they get a Goblin that makes their creatures
/// attack; not at yours.
#[test]
fn goblin_spymaster_hands_out_goblins() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::goblin_spymaster());
    g.step = TurnStep::End;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert!(named(&g, 1, "Goblin").is_empty(), "not on your own end step");
    g.active_player_idx = 1;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    let gob = named(&g, 1, "Goblin");
    assert_eq!(gob.len(), 1);
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    assert!(g.computed_permanent(bear).unwrap().keywords().contains(&Keyword::MustAttack));
}

/// {T}: {C} per card drawn this turn.
#[test]
fn kydele_taps_for_draws() {
    let mut g = pod(2);
    let k = g.add_card_to_battlefield(0, catalog::kydele_chosen_of_kruphix());
    g.clear_sickness(k);
    library(&mut g, 0, 3);
    run(&mut g, 0, &Effect::Draw { who: Selector::You, amount: Value::Const(3) });
    let before = g.players[0].mana_pool.total();
    g.perform_action(GameAction::ActivateAbility {
        card_id: k,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("tap");
    drain_stack(&mut g);
    assert_eq!(g.players[0].mana_pool.total(), before + 3);
}

/// Upkeep: a random discard; an opponent's discard: an Elf Warrior.
#[test]
fn nath_punishes_discards() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::nath_of_the_gilt_leaf());
    g.add_card_to_hand(1, catalog::island());
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Bool(true), DecisionAnswer::Bool(true)]));
    upkeep(&mut g);
    assert!(g.players[1].hand.is_empty());
    assert_eq!(named(&g, 0, "Elf Warrior").len(), 1);
}

/// From the graveyard: exile it, and everyone wheels.
#[test]
fn runehorn_hellkite_wheels_from_the_grave() {
    let mut g = pod(2);
    let h = g.add_card_to_graveyard(0, catalog::runehorn_hellkite());
    library(&mut g, 0, 8);
    library(&mut g, 1, 8);
    g.add_card_to_hand(1, catalog::island());
    activate(&mut g, 0, h, 0, None).expect("activate from the graveyard");
    assert!(g.exile.iter().any(|c| c.id == h));
    assert_eq!((g.players[0].hand.len(), g.players[1].hand.len()), (7, 7));
}

/// Copies of your and an opponent's graveyard spells, cast free.
#[test]
fn spelltwine_casts_both_copies() {
    let mut g = pod(2);
    let mine = g.add_card_to_graveyard(0, catalog::lightning_bolt());
    let theirs = g.add_card_to_graveyard(1, catalog::lightning_bolt());
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Bool(true), DecisionAnswer::Bool(true)]));
    let life = g.players[1].life;
    let s = g.add_card_to_hand(0, catalog::spelltwine());
    cast_with(&mut g, 0, s, Some(Target::Permanent(mine)), vec![Target::Permanent(theirs)]).expect("cast");
    assert_eq!(g.players[1].life, life - 6, "two Bolts at the opponent");
    assert!(g.exile.iter().any(|c| c.definition.name == "Spelltwine"));
}

/// {4}: scry, then a top land enters tapped; a nonland is drawn.
#[test]
fn thrasios_digs() {
    let mut g = pod(2);
    let t = g.add_card_to_battlefield(0, catalog::thrasios_triton_hero());
    g.add_card_to_library(0, catalog::forest());
    activate(&mut g, 0, t, 0, None).expect("activate");
    let f = named(&g, 0, "Forest");
    assert_eq!(f.len(), 1);
    assert!(g.battlefield_find(f[0]).unwrap().tapped);
    g.add_card_to_library(0, catalog::grizzly_bears());
    activate(&mut g, 0, t, 0, None).expect("activate");
    assert!(g.players[0].hand.iter().any(|c| c.definition.name == "Grizzly Bears"));
}

/// Each opponent takes their own land count.
#[test]
fn treacherous_terrain_counts_each_opponents_lands() {
    let mut g = pod(3);
    for _ in 0..3 {
        g.add_card_to_battlefield(1, catalog::island());
    }
    g.add_card_to_battlefield(2, catalog::island());
    let life = g.players[1].life;
    let s = g.add_card_to_hand(0, catalog::treacherous_terrain());
    cast(&mut g, 0, s, None).expect("cast");
    assert_eq!((g.players[1].life, g.players[2].life), (life - 3, life - 1));
}

/// The first spell each turn hits a random opponent for its mana value.
#[test]
fn vial_smasher_throws_the_first_spell() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::vial_smasher_the_fierce());
    let life = g.players[1].life;
    let angel = g.add_card_to_hand(0, catalog::serra_angel());
    cast(&mut g, 0, angel, None).expect("cast");
    assert_eq!(g.players[1].life, life - 5);
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    cast(&mut g, 0, bear, None).expect("cast");
    assert_eq!(g.players[1].life, life - 5, "the second spell doesn't");
}

/// Returns an instant, burns opposing creatures for its mana value.
#[test]
fn volcanic_vision_returns_and_burns() {
    let mut g = pod(2);
    let spell = g.add_card_to_graveyard(0, catalog::divination());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let s = g.add_card_to_hand(0, catalog::volcanic_vision());
    cast(&mut g, 0, s, Some(Target::Permanent(spell))).expect("cast");
    assert!(g.players[0].hand.iter().any(|c| c.id == spell));
    assert!(g.battlefield_find(bear).is_none(), "3 damage");
    assert!(g.battlefield_find(mine).is_some(), "yours are spared");
}

/// A Worm per land card in your graveyard.
#[test]
fn worm_harvest_counts_graveyard_lands() {
    let mut g = pod(2);
    for _ in 0..3 {
        g.add_card_to_graveyard(0, catalog::forest());
    }
    let s = g.add_card_to_hand(0, catalog::worm_harvest());
    cast(&mut g, 0, s, None).expect("cast");
    assert_eq!(named(&g, 0, "Worm").len(), 3);
}
