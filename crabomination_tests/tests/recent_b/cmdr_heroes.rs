//! Commander: the Turtle Power! precon (TMC, Heroes in a Half Shell,
//! `decks::cmdr_heroes`), and its primitives: mana from an artifact source
//! spent on a spell, a draw-caused trigger doubler (CR 603.2), Myriad-shaped
//! copies that are non-legendary and sacrificed, damage to your other
//! creatures as counters, and "each opponent you attacked this turn".

use crabomination::card::{CardId, CounterType, Supertype, Value};
use crabomination::catalog;
use crabomination::effect::{Effect, PlayerRef, Selector};
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn pod(n: usize) -> GameState {
    let mut g = multi_player_game(n);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    for seat in 0..n {
        for _ in 0..12 {
            g.add_card_to_library(seat, catalog::grizzly_bears());
        }
    }
    g
}

fn flood(g: &mut GameState, seat: usize) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[seat].mana_pool.add(c, 20);
    }
    g.players[seat].mana_pool.add_colorless(20);
}

fn cast_as(g: &mut GameState, seat: usize, id: CardId, targets: &[Target]) -> Result<(), String> {
    flood(g, seat);
    g.priority.player_with_priority = seat;
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: targets.first().cloned(),
        additional_targets: targets.iter().skip(1).cloned().collect(),
        mode: None,
        x_value: None,
    })
    .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn cast(g: &mut GameState, def: crabomination::card::CardDefinition) -> CardId {
    let id = g.add_card_to_hand(0, def);
    cast_as(g, 0, id, &[]).expect("castable");
    id
}

fn plus(g: &GameState, id: CardId) -> u32 {
    g.battlefield_find(id).map_or(0, |c| c.counter_count(CounterType::PlusOnePlusOne))
}

fn run(g: &mut GameState, seat: usize, effect: Effect) {
    let ctx = EffectContext::for_spell(seat, None, 0, 0);
    let evs = g.resolve_effect(&effect, &ctx).expect("resolves");
    g.dispatch_triggers_for_events(&evs);
    drain_stack(g);
}

fn named(g: &GameState, name: &str) -> usize {
    g.battlefield.iter().filter(|c| c.definition.name == name).count()
}

/// Attack, no blocks, through combat damage.
fn connect(g: &mut GameState, attacks: &[(CardId, usize)]) {
    for (a, _) in attacks {
        g.clear_sickness(*a);
    }
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = g.active_player_idx;
    g.perform_action(GameAction::DeclareAttackers(
        attacks.iter().map(|&(attacker, p)| Attack { attacker, target: AttackTarget::Player(p) }).collect(),
    ))
    .expect("attack");
    drain_stack(g);
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = attacks[0].1;
    g.perform_action(GameAction::DeclareBlockers(vec![])).expect("no blocks");
    while g.step != TurnStep::EndCombat {
        let _ = g.advance_step(Vec::new());
        drain_stack(g);
    }
}

// ── The commander ────────────────────────────────────────────────────────────

/// Heroes in a Half Shell: a Turtle's hit counters it up and draws a card.
#[test]
fn heroes_in_a_half_shell_grows_the_turtle_that_connected() {
    let mut g = pod(2);
    let heroes = g.add_card_to_battlefield(0, catalog::heroes_in_a_half_shell());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let hand = g.players[0].hand.len();
    connect(&mut g, &[(heroes, 1), (bear, 1)]);
    assert_eq!(plus(&g, heroes), 1);
    assert_eq!(plus(&g, bear), 0, "not a Mutant, Ninja or Turtle");
    assert_eq!(g.players[0].hand.len(), hand + 1, "one draw for the batch");
}

// ── Engine primitives ────────────────────────────────────────────────────────

/// Coin of Mastery: a creature enters with a counter for each mana from an
/// artifact source spent to cast it — Sol Ring's {C} counts, a Forest's {G}
/// doesn't.
#[test]
fn coin_of_mastery_counts_artifact_mana_spent() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::coin_of_mastery());
    let ring = g.add_card_to_battlefield(0, catalog::sol_ring());
    let forest = g.add_card_to_battlefield(0, catalog::forest());
    for id in [ring, forest] {
        g.perform_action(GameAction::ActivateAbility {
            card_id: id,
            ability_index: 0,
            target: None,
            additional_targets: vec![],
            x_value: None,
            mode: None,
        })
        .expect("tap for mana");
    }
    assert_eq!(g.players[0].mana_pool.artifact_amount(), 2, "Sol Ring's two");
    let bears = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.perform_action(GameAction::CastSpell { card_id: bears, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("cast with {C}{G}");
    drain_stack(&mut g);
    assert_eq!(plus(&g, bears), 1, "one mana from an artifact spent");
    // A creature cast with no artifact mana gets nothing.
    let other = cast(&mut g, catalog::grizzly_bears());
    assert_eq!(plus(&g, other), 0);
}

/// CR 603.2 — Krang: "you draw a card" (Baxter's counter) triggers twice;
/// Krang itself grows on a player's second draw of the turn.
#[test]
fn krang_doubles_draw_triggers() {
    let mut g = pod(2);
    let krang = g.add_card_to_battlefield(0, catalog::krang_the_all_powerful());
    let baxter = g.add_card_to_battlefield(0, catalog::baxter_fly_in_the_ointment());
    run(&mut g, 0, Effect::Draw { who: Selector::You, amount: Value::ONE });
    assert_eq!(plus(&g, baxter), 2, "Baxter's draw trigger fired twice");
    assert_eq!(plus(&g, krang), 0);
    run(&mut g, 1, Effect::Draw { who: Selector::You, amount: Value::Const(2) });
    assert_eq!(plus(&g, krang), 2, "an opponent's second draw, doubled (it is a draw trigger too)");
    assert_eq!(plus(&g, baxter), 2, "not Baxter's controller drawing");
}

/// Shredder: attacking one opponent in a four-seat pod, a non-legendary copy
/// attacks each of the other two; the hit halves each life total, rounded
/// up, and the copies are sacrificed at end of combat.
#[test]
fn shredder_sends_copies_at_every_other_opponent() {
    let mut g = pod(4);
    for p in 1..4 {
        g.players[p].life = 21;
    }
    let shredder = g.add_card_to_battlefield(0, catalog::shredder_shadow_master());
    connect(&mut g, &[(shredder, 1)]);
    for p in 1..4 {
        assert_eq!(g.players[p].life, 21 - 5 - 8, "seat {p}: 5 combat damage, then half of 16 rounded up");
    }
    assert_eq!(named(&g, "Shredder, Shadow Master"), 3, "the copies are still in combat");
    let _ = g.advance_step(Vec::new());
    drain_stack(&mut g);
    assert_eq!(named(&g, "Shredder, Shadow Master"), 1, "the copies were sacrificed at end of combat");
}

/// The copies are not legendary, so the legend rule leaves all three.
#[test]
fn shredders_copies_are_not_legendary() {
    let mut g = pod(3);
    let shredder = g.add_card_to_battlefield(0, catalog::shredder_shadow_master());
    g.clear_sickness(shredder);
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: shredder,
        target: AttackTarget::Player(1),
    }]))
    .expect("attack");
    drain_stack(&mut g);
    assert_eq!(named(&g, "Shredder, Shadow Master"), 2);
    let copy = g.battlefield.iter().find(|c| c.definition.name == "Shredder, Shadow Master" && c.id != shredder).unwrap();
    assert!(!copy.definition.supertypes.contains(&Supertype::Legendary));
    assert!(g.attacking.iter().any(|a| a.attacker == copy.id && a.target == AttackTarget::Player(2)));
}

/// Vigor: damage to another creature of yours becomes +1/+1 counters; Vigor
/// itself and an opponent's creature are hurt as usual.
#[test]
fn vigor_turns_damage_into_counters() {
    let mut g = pod(2);
    let vigor = g.add_card_to_battlefield(0, catalog::vigor());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    for id in [bear, vigor, theirs] {
        run(&mut g, 1, Effect::DealDamage { to: Selector::ExactObjects(vec![id]), amount: Value::Const(3) });
    }
    assert_eq!(plus(&g, bear), 3);
    assert_eq!(g.battlefield_find(bear).unwrap().damage, 0);
    assert_eq!(g.battlefield_find(vigor).unwrap().damage, 3, "only other creatures");
    assert!(g.battlefield_find(theirs).is_none(), "an opponent's creature isn't yours");
}

/// Fast Forward: {1} less per opponent attacked this turn.
#[test]
fn fast_forward_counts_opponents_attacked_this_turn() {
    let mut g = pod(4);
    g.players[0].attacked_players_this_turn = vec![1, 3];
    let bears: Vec<CardId> = (0..2).map(|_| g.add_card_to_battlefield(1, catalog::grizzly_bears())).collect();
    let ff = g.add_card_to_hand(0, catalog::fast_forward());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell { card_id: ff, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("{4}{R} less two is {2}{R}");
    drain_stack(&mut g);
    assert!(bears.iter().all(|&b| g.battlefield_find(b).unwrap().goaded_by.contains(&0)));
}

/// Dimension X Pizzasaur: after its two counters, the reflexive destroy
/// reaches a creature no bigger than the counters among your permanents.
#[test]
fn pizzasaur_destroys_up_to_your_counter_count() {
    let mut g = pod(2);
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let small = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let big = g.add_card_to_battlefield(1, catalog::serra_angel());
    let pizza = g.add_card_to_hand(0, catalog::dimension_x_pizzasaur());
    cast_as(&mut g, 0, pizza, &[]).expect("cast");
    assert_eq!(plus(&g, mine), 2);
    assert!(g.battlefield_find(small).is_none(), "mana value 2 ≤ 2 counters");
    assert!(g.battlefield_find(big).is_some(), "mana value 5 > 2 counters");
}

// ── Cards ────────────────────────────────────────────────────────────────────

/// Donatello: a token creation of yours adds a Mutagen, which grows a
/// creature at sorcery speed.
#[test]
fn donatello_adds_a_mutagen() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::donatello_the_brains());
    run(&mut g, 0, Effect::CreateToken {
        who: PlayerRef::You,
        count: Value::ONE,
        definition: std::sync::Arc::new(crabomination_base::tokens::food_token()),
    });
    assert_eq!(named(&g, "Mutagen"), 1);
    assert_eq!(named(&g, "Food"), 1);
}

/// Casey Jones: +1/+1 counters you put on your creature burn an opponent that
/// much.
#[test]
fn casey_jones_burns_for_your_counters() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::casey_jones_back_alley_brute());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let life = g.players[1].life;
    let ctx = EffectContext::for_ability(bear, 0, None);
    let evs = g
        .resolve_effect(
            &Effect::AddCounter {
                what: Selector::ExactObjects(vec![bear]),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(3),
            },
            &ctx,
        )
        .expect("counters");
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life - 3);
}

/// Rat King: a Rat as a nontoken creature of yours leaves, not a token; a
/// token pays for a draw.
#[test]
fn rat_king_replaces_the_fallen() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::rat_king_pale_piper());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    run(&mut g, 0, Effect::Destroy { what: Selector::ExactObjects(vec![bear]) });
    assert_eq!(named(&g, "Rat"), 1);
    let rat = g.battlefield.iter().find(|c| c.definition.name == "Rat").unwrap().id;
    run(&mut g, 0, Effect::Destroy { what: Selector::ExactObjects(vec![rat]) });
    assert_eq!(named(&g, "Rat"), 0, "a token leaving makes nothing");
}

/// Tokka & Rahzar: the first other nontoken creature of yours to leave each
/// turn grows it and makes a Treasure; the second does nothing.
#[test]
fn tokka_and_rahzar_trigger_once_a_turn() {
    let mut g = pod(2);
    let tr = g.add_card_to_battlefield(0, catalog::tokka_rahzar_unsupervised());
    for _ in 0..2 {
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        run(&mut g, 0, Effect::Destroy { what: Selector::ExactObjects(vec![bear]) });
    }
    assert_eq!(plus(&g, tr), 1);
    assert_eq!(named(&g, "Treasure"), 1);
}

/// Swift Demise: 1 damage, then each damaged creature you don't control dies —
/// yours survives.
#[test]
fn swift_demise_finishes_the_damaged() {
    let mut g = pod(2);
    let mine = g.add_card_to_battlefield(0, catalog::serra_angel());
    run(&mut g, 1, Effect::DealDamage { to: Selector::ExactObjects(vec![mine]), amount: Value::ONE });
    let hurt = g.add_card_to_battlefield(1, catalog::serra_angel());
    run(&mut g, 0, Effect::DealDamage { to: Selector::ExactObjects(vec![hurt]), amount: Value::ONE });
    let target = g.add_card_to_battlefield(1, catalog::serra_angel());
    let untouched = g.add_card_to_battlefield(1, catalog::serra_angel());
    let sd = g.add_card_to_hand(0, catalog::swift_demise());
    cast_as(&mut g, 0, sd, &[Target::Permanent(target)]).expect("cast");
    assert!(g.battlefield_find(target).is_none() && g.battlefield_find(hurt).is_none());
    assert!(g.battlefield_find(untouched).is_some() && g.battlefield_find(mine).is_some());
}

/// Continue?: creature cards of yours that died this turn come back; one
/// that was milled doesn't qualify.
#[test]
fn continue_returns_this_turns_dead() {
    let mut g = pod(2);
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    run(&mut g, 0, Effect::Destroy { what: Selector::ExactObjects(vec![bear]) });
    let milled = g.add_card_to_graveyard(0, catalog::serra_angel());
    let c = g.add_card_to_hand(0, catalog::continue_card());
    cast_as(&mut g, 0, c, &[Target::Permanent(bear)]).expect("cast");
    assert!(g.battlefield_find(bear).is_some());
    let again = g.add_card_to_hand(0, catalog::continue_card());
    assert!(cast_as(&mut g, 0, again, &[Target::Permanent(milled)]).is_err(), "never on the battlefield");
}

/// Big Apple, 3 a.m.: {5}, {T} makes a Rat per opponent.
#[test]
fn big_apple_makes_a_rat_per_opponent() {
    let mut g = pod(4);
    let land = g.add_card_to_battlefield(0, catalog::big_apple_3_a_m());
    flood(&mut g, 0);
    g.perform_action(GameAction::ActivateAbility {
        card_id: land,
        ability_index: 1,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("activate");
    drain_stack(&mut g);
    assert_eq!(named(&g, "Rat"), 3);
}

/// Game Over costs {2} less once any player is at half their starting life.
#[test]
fn game_over_is_cheaper_at_half_life() {
    let mut g = pod(2);
    let start = g.players[1].life;
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let go = g.add_card_to_hand(0, catalog::game_over());
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(1);
    assert!(
        g.perform_action(GameAction::CastSpell { card_id: go, target: None, additional_targets: vec![], mode: None, x_value: None })
            .is_err(),
        "{{3}}{{B}}{{B}} at full life"
    );
    g.players[1].life = start / 2;
    g.perform_action(GameAction::CastSpell { card_id: go, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("{1}{B}{B}");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none());
}

/// CR 702.102 — Double Jump // Flying Kick cast fused: the left half's slot
/// first, then Flying Kick's two. The jumper becomes a 5/5 flier and kicks
/// the opposing Serra Angel for 5; the right half used to read one target
/// and did nothing.
#[test]
fn double_jump_flying_kick_fused_uses_both_halves_targets() {
    let mut g = pod(2);
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let theirs = g.add_card_to_battlefield(1, catalog::serra_angel());
    let card = g.add_card_to_hand(0, catalog::double_jump_flying_kick());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastSplitFused {
        card_id: card,
        target: Some(Target::Permanent(mine)),
        additional_targets: vec![Target::Permanent(mine), Target::Permanent(theirs)],
        mode: None,
        x_value: None,
    })
    .expect("fused cast");
    drain_stack(&mut g);
    let cp = g.computed_permanent(mine).expect("still here");
    assert_eq!((cp.power, cp.toughness), (5, 5));
    assert!(cp.keywords().contains(&crabomination::card::Keyword::Flying));
    assert!(g.battlefield_find(theirs).is_none(), "5 damage to a 4/4");
}
