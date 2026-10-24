//! Commander: three-card gaps of EDHREC ranks 5501-6500
//! (`decks::cmdr_rank6k_c`).

use crabomination::card::{CardId, CounterType, Keyword};
use crabomination::catalog;
use crabomination::effect::Effect;
use crabomination::game::effects::EffectContext;
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

fn flood(g: &mut GameState) {
    for c in [
        Color::White,
        Color::Blue,
        Color::Black,
        Color::Red,
        Color::Green,
    ] {
        g.players[0].mana_pool.add(c, 20);
    }
    g.players[0].mana_pool.add_colorless(20);
}

fn ready(g: &mut GameState, seat: usize, def: crabomination::card::CardDefinition) -> CardId {
    let id = g.add_card_to_battlefield(seat, def);
    g.clear_sickness(id);
    id
}

fn act(g: &mut GameState, action: GameAction) {
    flood(g);
    g.priority.player_with_priority = 0;
    g.perform_action(action).expect("action");
    drain_stack(g);
}

fn cast(g: &mut GameState, id: CardId, target: Option<Target>) {
    act(
        g,
        GameAction::CastSpell {
            card_id: id,
            target,
            additional_targets: vec![],
            mode: None,
            x_value: None,
        },
    );
}

fn activate(g: &mut GameState, id: CardId, index: usize, target: Option<Target>) {
    act(
        g,
        GameAction::ActivateAbility {
            card_id: id,
            ability_index: index,
            target,
            additional_targets: vec![],
            x_value: None,
            mode: None,
        },
    );
}

fn named(g: &GameState, seat: usize, name: &str) -> usize {
    g.battlefield
        .iter()
        .filter(|c| c.controller == seat && c.definition.name == name)
        .count()
}

fn pt(g: &GameState, id: CardId) -> (i32, i32) {
    let c = g.computed_permanent(id).unwrap();
    (c.power, c.toughness)
}

fn plus(g: &GameState, id: CardId) -> u32 {
    g.battlefield_find(id)
        .unwrap()
        .counter_count(CounterType::PlusOnePlusOne)
}

fn resolve_at(
    g: &mut GameState,
    effect: &Effect,
    source: CardId,
    target: Option<Target>,
    mode: usize,
) {
    let mut ctx = EffectContext::for_spell(0, target, mode, 0);
    ctx.source = Some(source);
    let evs = g.resolve_effect(effect, &ctx).unwrap();
    g.dispatch_triggers_for_events(&evs);
    drain_stack(g);
}
/// CR 702.16 — the Wards give protection from their color.
#[test]
fn the_wards_grant_protection() {
    let mut g = pod(2);
    let hawk = ready(&mut g, 0, catalog::hawkeye_clint_barton());
    let w = g.add_card_to_hand(0, catalog::black_ward());
    cast(&mut g, w, Some(Target::Permanent(hawk)));
    let gw = g.add_card_to_hand(0, catalog::green_ward());
    cast(&mut g, gw, Some(Target::Permanent(hawk)));
    let kw = g.computed_permanent(hawk).unwrap().keywords().to_vec();
    assert!(
        kw.contains(&Keyword::Protection(Color::Black))
            && kw.contains(&Keyword::Protection(Color::Green))
    );
    assert!(kw.contains(&Keyword::Vigilance));
}

/// CR 702.193 — Hercules' and Gamma Grotesque's power-ups.
#[test]
fn hercules_and_gamma_grotesque_power_up() {
    let mut g = pod(2);
    let herc = ready(&mut g, 0, catalog::hercules_prince_of_power());
    let gg = ready(&mut g, 0, catalog::gamma_grotesque());
    activate(&mut g, herc, 0, None);
    let kw = g.computed_permanent(herc).unwrap().keywords().to_vec();
    assert!(kw.contains(&Keyword::Indestructible) && kw.contains(&Keyword::Haste));
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::island());
    }
    activate(&mut g, gg, 0, None);
    assert_eq!(plus(&g, gg), 3);
    assert_eq!(
        g.players[0].hand.len(),
        2,
        "Hercules and the Grotesque carry counters"
    );
}

/// CR 702.6 — +2 Mace is a +2/+2 Equipment.
#[test]
fn plus_two_mace_pumps() {
    let mut g = pod(2);
    let mace = ready(&mut g, 0, catalog::plus_two_mace());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    act(
        &mut g,
        GameAction::Equip {
            equipment: mace,
            target: bear,
        },
    );
    assert_eq!(pt(&g, bear), (4, 4));
}

/// CR 506.4 — Blessed Reversal gains 3 per creature attacking you.
#[test]
fn blessed_reversal_counts_attackers() {
    let mut g = pod(2);
    g.active_player_idx = 1;
    let a = ready(&mut g, 1, catalog::grizzly_bears());
    let b = ready(&mut g, 1, catalog::grizzly_bears());
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::DeclareAttackers(vec![
        Attack {
            attacker: a,
            target: AttackTarget::Player(0),
        },
        Attack {
            attacker: b,
            target: AttackTarget::Player(0),
        },
    ]))
    .expect("attack");
    let br = g.add_card_to_hand(0, catalog::blessed_reversal());
    cast(&mut g, br, None);
    assert_eq!(g.players[0].life, 26);
}

/// CR 119.10 — Doctor Jane Foster returns to the battlefield after life gain,
/// else to hand.
#[test]
fn doctor_jane_foster_returns_by_life_gain() {
    let mut g = pod(2);
    let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let jane = ready(&mut g, 0, catalog::doctor_jane_foster());
    let etb = catalog::doctor_jane_foster().triggered_abilities[0]
        .effect
        .clone();
    resolve_at(&mut g, &etb, jane, Some(Target::Permanent(bear)), 0);
    assert!(
        g.players[0]
            .hand
            .iter()
            .any(|c| c.definition.name == "Grizzly Bears")
    );
    let elf = g.add_card_to_graveyard(0, catalog::llanowar_elves());
    let hh = g.add_card_to_hand(0, catalog::healing_hands());
    g.add_card_to_library(0, catalog::island());
    cast(&mut g, hh, Some(Target::Player(0)));
    assert_eq!(g.players[0].life, 24);
    resolve_at(&mut g, &etb, jane, Some(Target::Permanent(elf)), 0);
    assert_eq!(named(&g, 0, "Llanowar Elves"), 1);
}

/// CR 615.7 — Heal shields 1 damage and draws next upkeep.
#[test]
fn heal_prevents_and_draws_later() {
    let mut g = pod(2);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let heal = g.add_card_to_hand(0, catalog::heal());
    cast(&mut g, heal, Some(Target::Permanent(bear)));
    let bolt = Effect::DealDamage {
        to: crabomination::effect::Selector::Target(0),
        amount: crabomination::effect::Value::Const(2),
    };
    resolve_at(&mut g, &bolt, bear, Some(Target::Permanent(bear)), 0);
    assert!(g.battlefield_find(bear).is_some(), "1 of the 2 prevented");
}

/// CR 702.100 — Darwin evolves and cashes two counters for indestructible.
#[test]
fn darwin_evolves() {
    let mut g = pod(2);
    let d = ready(&mut g, 0, catalog::darwin_adaptive_mutant());
    for _ in 0..2 {
        let angel = g.add_card_to_hand(0, catalog::serra_angel());
        cast(&mut g, angel, None);
    }
    assert_eq!(plus(&g, d), 2);
    activate(&mut g, d, 0, None);
    assert!(
        g.computed_permanent(d)
            .unwrap()
            .keywords()
            .contains(&Keyword::Indestructible)
    );
}

/// CR 207.2c — Beorn's Hospitality counters on landfall and can stand up.
#[test]
fn beorns_hospitality_landfalls() {
    let mut g = pod(2);
    let bh = ready(&mut g, 0, catalog::beorns_hospitality());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let land = g.add_card_to_hand(0, catalog::forest());
    g.perform_action(GameAction::PlayLand(land)).expect("land");
    drain_stack(&mut g);
    assert_eq!(plus(&g, bear), 1);
    activate(&mut g, bh, 0, None);
    assert_eq!(pt(&g, bh), (1, 1));
    // The gained "P/T equal to the lands you control" keeps counting.
    ready(&mut g, 0, catalog::forest());
    assert_eq!(pt(&g, bh), (2, 2));
}

/// CR 702.27 — Wurmcalling makes an X/X and buys itself back.
#[test]
fn wurmcalling_makes_an_x_x() {
    let mut g = pod(2);
    let wc = g.add_card_to_hand(0, catalog::wurmcalling());
    flood(&mut g);
    g.perform_action(GameAction::CastSpell {
        card_id: wc,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: Some(4),
    })
    .expect("cast");
    drain_stack(&mut g);
    let wurm = g
        .battlefield
        .iter()
        .find(|c| c.definition.name == "Wurm")
        .expect("wurm")
        .id;
    assert_eq!(pt(&g, wurm), (4, 4));
}
