//! Commander: Magar, Narci and Yennett's last gaps (`decks::cmdr_edhrec19`).

use crabomination::card::{CardId, CounterType, Keyword, Value};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::effect::{Effect, Selector};
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn pod(n: usize) -> GameState {
    let mut g = multi_player_game(n);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g
}

fn flood(g: &mut GameState, seat: usize) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[seat].mana_pool.add(c, 20);
    }
    g.players[seat].mana_pool.add_colorless(20);
}

fn cast(g: &mut GameState, id: CardId, targets: &[Target]) -> Result<(), String> {
    flood(g, 0);
    g.priority.player_with_priority = 0;
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

fn ready(g: &mut GameState, seat: usize, def: crabomination::card::CardDefinition) -> CardId {
    let id = g.add_card_to_battlefield(seat, def);
    g.clear_sickness(id);
    id
}

fn run(g: &mut GameState, effect: &Effect) {
    let ctx = EffectContext::for_spell(0, None, 0, 0);
    let evs = g.resolve_effect(effect, &ctx).unwrap();
    g.dispatch_triggers_for_events(&evs);
    drain_stack(g);
}

/// Add one lore counter to `saga` the way an effect would, firing its chapter.
fn lore(g: &mut GameState, saga: CardId) {
    run(
        g,
        &Effect::AddCounter { what: Selector::ExactObjects(vec![saga]), kind: CounterType::Lore, amount: Value::ONE },
    );
}

fn attack_and_hit(g: &mut GameState, attacker: CardId, defender: usize) {
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker,
        target: AttackTarget::Player(defender),
    }]))
    .expect("attack");
    drain_stack(g);
    g.step = TurnStep::CombatDamage;
    let ev = g.resolve_combat().expect("damage");
    g.dispatch_triggers_for_events(&ev);
    drain_stack(g);
}

fn in_hand(g: &GameState, seat: usize, id: CardId) -> bool {
    g.players[seat].hand.iter().any(|c| c.id == id)
}

fn in_exile(g: &GameState, id: CardId) -> bool {
    g.exile.iter().any(|c| c.id == id)
}

/// Magar's puppet: a face-down 3/3 that, on combat damage to a player, casts
/// a free copy of the instant it hides; leaving the battlefield exiles it
/// (2022-10-07 rulings).
#[test]
fn magar_puppet_recasts_its_noted_spell() {
    let mut g = pod(2);
    let magar = ready(&mut g, 0, catalog::magar_of_the_magic_strings());
    let bolt = g.add_card_to_graveyard(0, catalog::lightning_bolt());
    flood(&mut g, 0);
    g.perform_action(GameAction::ActivateAbility {
        card_id: magar,
        ability_index: 0,
        target: Some(Target::Permanent(bolt)),
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("activate");
    drain_stack(&mut g);
    let puppet = g.battlefield_find(bolt).expect("the bolt is on the battlefield");
    assert!(puppet.face_down);
    let cp = g.computed_permanent(bolt).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 3));
    g.clear_sickness(bolt);
    let life = g.players[1].life;
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    attack_and_hit(&mut g, bolt, 1);
    assert_eq!(g.players[1].life, life - 6, "3 combat damage, then a free copy of Lightning Bolt");
    run(&mut g, &Effect::Destroy { what: Selector::ExactObjects(vec![bolt]) });
    assert!(in_exile(&g, bolt), "a puppet that would leave is exiled instead");
}

/// Sorin's Vengeance drains 10; Worst Fears queues control of the target's
/// next turn (CR 723.1) and exiles itself.
#[test]
fn magar_sorceries() {
    let mut g = pod(2);
    let sorin = g.add_card_to_hand(0, catalog::sorins_vengeance());
    cast(&mut g, sorin, &[Target::Player(1)]).expect("sorin");
    assert_eq!((g.players[0].life, g.players[1].life), (30, 10));
    let fears = g.add_card_to_hand(0, catalog::worst_fears());
    cast(&mut g, fears, &[Target::Player(1)]).expect("fears");
    assert!(g.pending_player_control.contains(&(1, 0)));
    assert!(in_exile(&g, fears));
}

/// Azra Oddsmaker: at your beginning of combat, a discard sets up "draw two
/// when that creature connects" — the creature chosen on resolution (CR
/// 608.2d), and the bot's pick is its own, not a bigger opposing body.
#[test]
fn azra_oddsmaker_draws_on_the_hit() {
    let mut g = pod(2);
    for _ in 0..4 {
        g.add_card_to_library(0, catalog::island());
    }
    let azra = ready(&mut g, 0, catalog::azra_oddsmaker());
    ready(&mut g, 1, catalog::serra_angel());
    g.add_card_to_hand(0, catalog::island());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    g.step = TurnStep::BeginCombat;
    g.fire_step_triggers(TurnStep::BeginCombat);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), 0, "the card was discarded");
    attack_and_hit(&mut g, azra, 1);
    assert_eq!(g.players[0].hand.len(), 2);
}

/// Tale of Tinúviel: I — indestructible while the Saga stays; II — reanimate;
/// III — lifelink.
#[test]
fn tale_of_tinuviel_chapters() {
    let mut g = pod(2);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let dead = g.add_card_to_graveyard(0, catalog::hill_giant());
    let tale = g.add_card_to_hand(0, catalog::tale_of_tinuviel());
    cast(&mut g, tale, &[Target::Permanent(bear)]).expect("cast");
    assert!(g.computed_permanent(bear).unwrap().keywords().contains(&Keyword::Indestructible));
    lore(&mut g, tale);
    assert!(g.battlefield_find(dead).is_some(), "chapter II returned the giant");
    lore(&mut g, tale);
    g.check_state_based_actions();
    assert!(g.battlefield_find(tale).is_none(), "sacrificed after III");
    assert!(
        !g.computed_permanent(bear).unwrap().keywords().contains(&Keyword::Indestructible),
        "the indestructible ended with the Saga"
    );
}

/// The Aesir Escape Valhalla: the exiled card's mana value is the life, then
/// the counters, then both return to hand.
#[test]
fn aesir_escape_valhalla_round_trip() {
    let mut g = pod(2);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let giant = g.add_card_to_graveyard(0, catalog::hill_giant());
    let saga = g.add_card_to_hand(0, catalog::the_aesir_escape_valhalla());
    cast(&mut g, saga, &[]).expect("cast");
    assert!(in_exile(&g, giant));
    assert_eq!(g.players[0].life, 24, "Hill Giant's mana value is 4");
    lore(&mut g, saga);
    assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne), 4);
    lore(&mut g, saga);
    assert!(in_hand(&g, 0, giant) && in_hand(&g, 0, saga), "both went home");
}

/// The Aesir Escape Valhalla's chapter I exile is the controller's choice, not
/// the priciest card — and chapter II reads the mana value of the one chosen.
#[test]
fn aesir_escape_valhalla_exiles_the_chosen_card() {
    let mut g = pod(2);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let giant = g.add_card_to_graveyard(0, catalog::hill_giant());
    let elf = g.add_card_to_graveyard(0, catalog::llanowar_elves());
    let saga = g.add_card_to_hand(0, catalog::the_aesir_escape_valhalla());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Cards(vec![elf])]));
    cast(&mut g, saga, &[]).expect("cast");
    assert!(in_exile(&g, elf) && !in_exile(&g, giant));
    assert_eq!(g.players[0].life, 21, "Llanowar Elves' mana value is 1");
    lore(&mut g, saga);
    assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
}

/// The Creation of Avacyn: tutor to exile, pay its mana value in life if
/// it's a creature, then put it onto the battlefield.
#[test]
fn creation_of_avacyn_deploys_the_tutored_creature() {
    let mut g = pod(2);
    let giant = g.add_card_to_library(0, catalog::hill_giant());
    let saga = g.add_card_to_hand(0, catalog::the_creation_of_avacyn());
    cast(&mut g, saga, &[]).expect("cast");
    assert!(in_exile(&g, giant));
    lore(&mut g, saga);
    assert_eq!(g.players[0].life, 16);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    lore(&mut g, saga);
    assert!(g.battlefield_find(giant).is_some(), "giant in hand: {}, exile: {}", in_hand(&g, 0, giant), in_exile(&g, giant));
}

/// The World Spell's chapter III puts up to two non-Saga permanent cards from
/// hand onto the battlefield.
#[test]
fn world_spell_deploys_two_from_hand() {
    let mut g = pod(2);
    let saga = ready(&mut g, 0, catalog::the_world_spell());
    let a = g.add_card_to_hand(0, catalog::hill_giant());
    let b = g.add_card_to_hand(0, catalog::grizzly_bears());
    let other_saga = g.add_card_to_hand(0, catalog::the_creation_of_avacyn());
    g.battlefield_find_mut(saga).unwrap().add_counters(CounterType::Lore, 2);
    lore(&mut g, saga);
    assert!(g.battlefield_find(a).is_some() && g.battlefield_find(b).is_some());
    assert!(in_hand(&g, 0, other_saga), "a Saga card isn't eligible");
}

/// Hidetsugu and Kairi: ETB nets one card; its death drains the top card's
/// mana value from the target opponent and casts an instant free.
#[test]
fn hidetsugu_and_kairi_enter_and_die() {
    let mut g = pod(2);
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::island());
    }
    let hk = g.add_card_to_hand(0, catalog::hidetsugu_and_kairi());
    cast(&mut g, hk, &[]).expect("cast");
    assert_eq!(g.players[0].hand.len(), 1, "draw three, put two back");
    assert_eq!(g.players[0].library.len(), 2);
    let mut g = pod(2);
    let hk = ready(&mut g, 0, catalog::hidetsugu_and_kairi());
    g.add_card_to_library(0, catalog::hill_giant());
    run(&mut g, &Effect::Destroy { what: Selector::ExactObjects(vec![hk]) });
    assert_eq!(g.players[1].life, 16, "Hill Giant's mana value");
}

/// Scholar of the Lost Trove recasts an instant from the graveyard free, and
/// the instant is exiled afterwards.
#[test]
fn scholar_of_the_lost_trove_recasts_and_exiles() {
    let mut g = pod(2);
    let bolt = g.add_card_to_graveyard(0, catalog::lightning_bolt());
    let scholar = g.add_card_to_hand(0, catalog::scholar_of_the_lost_trove());
    cast(&mut g, scholar, &[]).expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 17);
    assert!(in_exile(&g, bolt));
}

/// Vesuvan Drifter becomes the top creature card until end of turn, and keeps
/// flying.
#[test]
fn vesuvan_drifter_copies_the_top_creature() {
    let mut g = pod(2);
    g.add_card_to_library(0, catalog::hill_giant());
    let drifter = ready(&mut g, 0, catalog::vesuvan_drifter());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    g.step = TurnStep::BeginCombat;
    g.fire_step_triggers(TurnStep::BeginCombat);
    drain_stack(&mut g);
    let cp = g.computed_permanent(drifter).unwrap();
    assert_eq!(cp.def.name, "Hill Giant");
    assert!(cp.keywords().contains(&Keyword::Flying));
}

/// Brilliant Ultimatum: the chosen pile's spells are cast free as it resolves
/// (2008-10-01 ruling); the other pile stays exiled.
#[test]
fn brilliant_ultimatum_casts_a_pile_free() {
    let mut g = pod(2);
    for _ in 0..5 {
        g.add_card_to_library(0, catalog::grizzly_bears());
    }
    let ult = g.add_card_to_hand(0, catalog::brilliant_ultimatum());
    cast(&mut g, ult, &[]).expect("cast");
    let bears = g.battlefield.iter().filter(|c| c.definition.name == "Grizzly Bears").count();
    let exiled = g.exile.iter().filter(|c| c.definition.name == "Grizzly Bears").count();
    assert!(bears >= 2, "a pile of at least two came down free");
    assert_eq!(bears + exiled, 5);
}
