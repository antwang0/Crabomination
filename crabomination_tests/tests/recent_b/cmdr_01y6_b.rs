//! Commander: the 2026-10-25 (`01Y6ZRFx`) Irini Sengir closers
//! (`decks::cmdr_01y6_b`), and the knight pair the Teroh / Irini decks share.

use crabomination::card::{CardId, CounterType, Keyword};
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

fn cast(g: &mut GameState, id: CardId, target: Option<Target>) -> Result<(), GameError> {
    g.perform_action(GameAction::CastSpell { card_id: id, target, additional_targets: vec![], mode: None, x_value: None })?;
    drain_stack(g);
    Ok(())
}

fn activate(g: &mut GameState, id: CardId, idx: usize) -> Result<(), GameError> {
    g.perform_action(GameAction::ActivateAbility {
        card_id: id,
        ability_index: idx,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })?;
    drain_stack(g);
    Ok(())
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

/// Order of the White Shield / Knight of Stromgald: protection from the
/// enemy colour, a first-strike pump and a +1/+0 pump in their own colour.
#[test]
fn the_shield_knights_pump_themselves() {
    for (def, color, prot) in [
        (catalog::order_of_the_white_shield(), Color::White, Color::Black),
        (catalog::knight_of_stromgald(), Color::Black, Color::White),
    ] {
        let mut g = pod(2);
        let k = ready(&mut g, 0, def);
        assert!(g.permanent_has_keyword(k, &Keyword::Protection(prot)));
        g.players[0].mana_pool.add(color, 3);
        activate(&mut g, k, 0).expect("first strike");
        activate(&mut g, k, 1).expect("+1/+0");
        assert!(g.permanent_has_keyword(k, &Keyword::FirstStrike));
        assert_eq!(g.computed_permanent(k).unwrap().power, 3);
    }
}

/// Dunerider Outlaw grows at the end step after it hits an opponent, not
/// after a quiet turn.
#[test]
fn dunerider_outlaw_grows_after_connecting() {
    for connects in [true, false] {
        let mut g = pod(2);
        let o = ready(&mut g, 0, catalog::dunerider_outlaw());
        if connects {
            g.step = TurnStep::DeclareAttackers;
            g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: o, target: AttackTarget::Player(1) }]))
                .expect("attack");
            drain_stack(&mut g);
            g.step = TurnStep::DeclareBlockers;
            g.priority.player_with_priority = 1;
            g.perform_action(GameAction::DeclareBlockers(vec![])).expect("no blocks");
            g.step = TurnStep::CombatDamage;
            let ev = g.resolve_combat().expect("damage");
            g.dispatch_triggers_for_events(&ev);
            drain_stack(&mut g);
        }
        g.step = TurnStep::PostCombatMain;
        g.priority.player_with_priority = 0;
        while g.step != TurnStep::End {
            g.perform_action(GameAction::PassPriority).expect("pass");
        }
        drain_stack(&mut g);
        assert_eq!(g.battlefield_find(o).unwrap().counter_count(CounterType::PlusOnePlusOne), connects as u32);
    }
}

/// Lifebane Zombie exiles a green creature card from the opponent's hand and
/// leaves a red one.
#[test]
fn lifebane_zombie_exiles_a_green_or_white_creature_card() {
    let mut g = pod(2);
    let bear = g.add_card_to_hand(1, catalog::grizzly_bears());
    let piker = g.add_card_to_hand(1, catalog::goblin_piker());
    let z = g.add_card_to_hand(0, catalog::lifebane_zombie());
    g.players[0].mana_pool.add(Color::Black, 3);
    cast(&mut g, z, None).expect("cast");
    assert!(g.exile.iter().any(|c| c.id == bear), "the Bears exiled");
    assert!(g.players[1].hand.iter().any(|c| c.id == piker), "the Piker kept");
}

/// Curse of Wizardry (red): any player's red spell costs its caster 1 life.
#[test]
fn curse_of_wizardry_drains_the_chosen_colors_casters() {
    let mut g = pod(3);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Color(Color::Red)]));
    let curse = g.add_card_to_hand(0, catalog::curse_of_wizardry());
    g.players[0].mana_pool.add(Color::Black, 4);
    cast(&mut g, curse, None).expect("cast");
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.players[1].mana_pool.add(Color::Red, 1);
    let life = g.players[1].life;
    cast(&mut g, bolt, Some(Target::Player(2))).expect("bolt");
    assert_eq!(g.players[1].life, life - 1);
}

/// Dystopia: the upkeep's active player sacrifices a green or white
/// permanent; others keep theirs.
#[test]
fn dystopia_eats_a_green_or_white_permanent_each_upkeep() {
    let mut g = pod(3);
    g.add_card_to_battlefield(0, catalog::dystopia());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let other = g.add_card_to_battlefield(2, catalog::savannah_lions());
    to_upkeep(&mut g, 1);
    assert!(g.battlefield_find(bear).is_none());
    assert!(g.battlefield_find(other).is_some());
}

/// CR 601.2f / 602.2b — Gloom: a white spell costs {3} more; a white
/// enchantment's activated ability too, a white creature's does not.
#[test]
fn gloom_taxes_white_spells_and_white_enchantment_abilities() {
    let mut g = pod(2);
    g.add_card_to_battlefield(1, catalog::gloom());
    let lions = g.add_card_to_hand(0, catalog::savannah_lions());
    g.players[0].mana_pool.add(Color::White, 1);
    assert!(cast(&mut g, lions, None).is_err(), "{{W}} is not enough");
    g.players[0].mana_pool.add_colorless(3);
    cast(&mut g, lions, None).expect("{W} + {3}");
    // Order of the White Shield's own abilities are a creature's: untaxed.
    let o = ready(&mut g, 0, catalog::order_of_the_white_shield());
    g.players[0].mana_pool.empty();
    g.players[0].mana_pool.add(Color::White, 1);
    activate(&mut g, o, 0).expect("a creature's ability is untaxed");
    let mesa = g.add_card_to_battlefield(0, catalog::sacred_mesa());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    assert!(activate(&mut g, mesa, 0).is_err(), "{{1}}{{W}} is not enough");
    g.players[0].mana_pool.add_colorless(3);
    activate(&mut g, mesa, 0).expect("{1}{W} + {3}");
}

/// Leshrac's Sigil: a green spell lets you pay {B}{B} to strip a card; its
/// own {B}{B} returns it to hand.
#[test]
fn leshracs_sigil_punishes_green_spells() {
    let mut g = pod(2);
    let sigil = g.add_card_to_battlefield(0, catalog::leshracs_sigil());
    g.players[0].mana_pool.add(Color::Black, 2);
    let kept = g.add_card_to_hand(1, catalog::island());
    let growth = g.add_card_to_hand(1, catalog::giant_growth());
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.players[1].mana_pool.add(Color::Green, 1);
    cast(&mut g, growth, Some(Target::Permanent(bear))).expect("growth");
    assert!(!g.players[1].hand.iter().any(|c| c.id == kept), "the Island discarded");
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Black, 2);
    activate(&mut g, sigil, 0).expect("bounce");
    assert!(g.players[0].hand.iter().any(|c| c.id == sigil));
}

/// Tourach's Chant: a Forest entering under a player with no creature to
/// shrink costs them 3 life.
#[test]
fn tourachs_chant_burns_forest_drops() {
    let mut g = pod(3);
    g.add_card_to_battlefield(0, catalog::tourachs_chant());
    let forest = g.add_card_to_hand(2, catalog::forest());
    g.active_player_idx = 2;
    g.priority.player_with_priority = 2;
    let life = g.players[2].life;
    g.perform_action(GameAction::PlayLand(forest)).expect("play");
    drain_stack(&mut g);
    assert_eq!(g.players[2].life, life - 3);
}

/// Noxious Grasp only takes a green or white target; Virtue's Ruin only
/// white creatures.
#[test]
fn noxious_grasp_and_virtues_ruin_pick_their_colors() {
    let mut g = pod(2);
    let vamp = g.add_card_to_battlefield(1, catalog::sengir_vampire());
    let lions = g.add_card_to_battlefield(1, catalog::savannah_lions());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let grasp = g.add_card_to_hand(0, catalog::noxious_grasp());
    g.players[0].mana_pool.add(Color::Black, 2);
    assert!(cast(&mut g, grasp, Some(Target::Permanent(vamp))).is_err(), "black is no target");
    let life = g.players[0].life;
    cast(&mut g, grasp, Some(Target::Permanent(bear))).expect("green");
    assert_eq!(g.players[0].life, life + 1);
    let ruin = g.add_card_to_hand(0, catalog::virtues_ruin());
    g.players[0].mana_pool.add(Color::Black, 3);
    cast(&mut g, ruin, None).expect("ruin");
    assert!(g.battlefield_find(lions).is_none());
    assert!(g.battlefield_find(vamp).is_some());
}
