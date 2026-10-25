//! Commander: the 2026-10-25 (`01Y6ZRFx`) Major Teroh closers
//! (`decks::cmdr_01y6_a`).

use crabomination::card::{CardId, CardType};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
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

fn ready(g: &mut GameState, seat: usize, def: crabomination::card::CardDefinition) -> CardId {
    let id = g.add_card_to_battlefield(seat, def);
    g.clear_sickness(id);
    id
}

fn cast_at(g: &mut GameState, id: CardId, target: Target) {
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(target),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(g);
}

fn to_upkeep(g: &mut GameState, seat: usize) {
    g.active_player_idx = seat;
    g.step = TurnStep::Untap;
    g.priority.player_with_priority = seat;
    while g.step != TurnStep::Upkeep {
        g.perform_action(GameAction::PassPriority).expect("pass priority");
    }
    drain_stack(g);
}

/// `attacker` (seat `from`) attacks seat `to`, who blocks with `blocker`.
fn fight(g: &mut GameState, from: usize, attacker: CardId, to: usize, blocker: Option<CardId>) {
    g.active_player_idx = from;
    g.priority.player_with_priority = from;
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker, target: AttackTarget::Player(to) }]))
        .expect("attack");
    drain_stack(g);
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = to;
    g.perform_action(GameAction::DeclareBlockers(blocker.map(|b| vec![(b, attacker)]).unwrap_or_default()))
        .expect("blocks");
    drain_stack(g);
    g.step = TurnStep::CombatDamage;
    let ev = g.resolve_combat().expect("damage");
    g.dispatch_triggers_for_events(&ev);
    drain_stack(g);
}

/// Kormus Bell: every Swamp, whoever controls it, is a 1/1 black creature land.
#[test]
fn kormus_bell_animates_every_swamp_black() {
    let mut g = pod(3);
    g.add_card_to_battlefield(0, catalog::kormus_bell());
    let swamp = g.add_card_to_battlefield(2, catalog::swamp());
    let plains = g.add_card_to_battlefield(2, catalog::plains());
    let c = g.computed_permanent(swamp).unwrap();
    assert!(c.card_types().contains(&CardType::Creature) && c.card_types().contains(&CardType::Land));
    assert_eq!((c.power, c.toughness), (1, 1));
    assert!(c.colors.contains(Color::Black));
    assert!(!g.computed_permanent(plains).unwrap().card_types().contains(&CardType::Creature));
}

/// Drought taxes its own controller's black spells too, and its upkeep
/// asks {W}{W}.
#[test]
fn drought_taxes_every_player_and_costs_ww_to_keep() {
    let mut g = pod(2);
    let drought = g.add_card_to_battlefield(0, catalog::drought());
    let hymn = g.add_card_to_hand(0, catalog::hymn_to_tourach());
    g.players[0].mana_pool.add(Color::Black, 3);
    let cast = g.perform_action(GameAction::CastSpell {
        card_id: hymn,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    });
    assert!(cast.is_err(), "no Swamps for the two black symbols");
    g.players[0].mana_pool.empty();
    to_upkeep(&mut g, 0);
    assert!(g.battlefield_find(drought).is_none(), "no {{W}}{{W}}: sacrificed");
}

/// Karma hits each upkeep's active player for their own Swamps only.
#[test]
fn karma_burns_the_active_player_per_swamp() {
    let mut g = pod(3);
    g.add_card_to_battlefield(0, catalog::karma());
    for _ in 0..3 {
        g.add_card_to_battlefield(1, catalog::swamp());
    }
    g.add_card_to_battlefield(2, catalog::swamp());
    let life: Vec<i32> = g.players.iter().map(|p| p.life).collect();
    to_upkeep(&mut g, 1);
    assert_eq!(g.players[1].life, life[1] - 3);
    assert_eq!((g.players[0].life, g.players[2].life), (life[0], life[2]));
}

/// Mangara's Equity (black chosen): a black attacker that hits you takes
/// the same damage back; a red one does not.
#[test]
fn mangaras_equity_hits_back_the_chosen_color() {
    for (attacker, punished) in [(catalog::sengir_vampire(), true), (catalog::goblin_piker(), false)] {
        let mut g = pod(2);
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Color(Color::Black)]));
        let eq = g.add_card_to_hand(0, catalog::mangaras_equity());
        g.players[0].mana_pool.add(Color::White, 3);
        g.perform_action(GameAction::CastSpell { card_id: eq, target: None, additional_targets: vec![], mode: None, x_value: None })
            .expect("cast");
        drain_stack(&mut g);
        assert_eq!(g.battlefield_find(eq).and_then(|c| c.chosen_color), Some(Color::Black));
        let a = ready(&mut g, 1, attacker);
        fight(&mut g, 1, a, 0, None);
        assert_eq!(g.battlefield_find(a).is_none(), punished, "{}", g.players[0].life);
    }
}

/// Mangara's Equity also covers a white creature you control: a black
/// creature it blocks takes the damage it dealt.
#[test]
fn mangaras_equity_covers_your_white_creatures() {
    let mut g = pod(2);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Color(Color::Black)]));
    let eq = g.add_card_to_hand(0, catalog::mangaras_equity());
    g.players[0].mana_pool.add(Color::White, 3);
    g.perform_action(GameAction::CastSpell { card_id: eq, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("cast");
    drain_stack(&mut g);
    // Savannah Lions (2/1) blocks Erg Raiders (2/3): the Lions' 2 alone
    // leave it alive, the 2 back finish it.
    let lions = ready(&mut g, 0, catalog::savannah_lions());
    let a = ready(&mut g, 1, catalog::erg_raiders());
    fight(&mut g, 1, a, 0, Some(lions));
    assert!(g.battlefield_find(a).is_none());
}

/// Royal Decree pings a tapped Swamp's controller; a Plains is untouched.
#[test]
fn royal_decree_pings_on_swamp_taps() {
    let mut g = pod(3);
    g.add_card_to_battlefield(0, catalog::royal_decree());
    let swamp = g.add_card_to_battlefield(2, catalog::swamp());
    let plains = g.add_card_to_battlefield(2, catalog::plains());
    let life = g.players[2].life;
    for land in [swamp, plains] {
        g.active_player_idx = 2;
        g.priority.player_with_priority = 2;
        g.perform_action(GameAction::ActivateAbility {
            card_id: land,
            ability_index: 0,
            target: None,
            additional_targets: vec![],
            x_value: None,
            mode: None,
        })
        .expect("tap for mana");
        drain_stack(&mut g);
    }
    assert_eq!(g.players[2].life, life - 1, "one ping, for the Swamp");
}

/// Inquisitor's Snare destroys a black attacker; a green one lives but deals
/// nothing.
#[test]
fn inquisitors_snare_destroys_black_and_muzzles_the_rest() {
    for (def, dies) in [(catalog::sengir_vampire(), true), (catalog::grizzly_bears(), false)] {
        let mut g = pod(2);
        let a = ready(&mut g, 1, def);
        g.active_player_idx = 1;
        g.priority.player_with_priority = 1;
        g.step = TurnStep::DeclareAttackers;
        g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: a, target: AttackTarget::Player(0) }]))
            .expect("attack");
        drain_stack(&mut g);
        let snare = g.add_card_to_hand(0, catalog::inquisitors_snare());
        g.players[0].mana_pool.add(Color::White, 2);
        g.priority.player_with_priority = 0;
        cast_at(&mut g, snare, Target::Permanent(a));
        assert_eq!(g.battlefield_find(a).is_none(), dies);
        if !dies {
            let life = g.players[0].life;
            g.step = TurnStep::DeclareBlockers;
            g.perform_action(GameAction::DeclareBlockers(vec![])).expect("no blocks");
            g.step = TurnStep::CombatDamage;
            let ev = g.resolve_combat().expect("damage");
            g.dispatch_triggers_for_events(&ev);
            assert_eq!(g.players[0].life, life, "prevented");
        }
    }
}

/// Starlight counts the targeted opponent's black creatures, not the table's.
#[test]
fn starlight_counts_one_opponents_black_creatures() {
    let mut g = pod(3);
    for _ in 0..2 {
        g.add_card_to_battlefield(1, catalog::sengir_vampire());
    }
    g.add_card_to_battlefield(2, catalog::sengir_vampire());
    g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let life = g.players[0].life;
    let s = g.add_card_to_hand(0, catalog::starlight());
    g.players[0].mana_pool.add(Color::White, 2);
    cast_at(&mut g, s, Target::Player(1));
    assert_eq!(g.players[0].life, life + 6);
}
