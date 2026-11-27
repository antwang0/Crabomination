//! Whtz's discount and the chooser / Doctor pair closers
//! (`decks::cmdr_01wr4b_a`).

use crabomination::card::CardId;
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::effect::{Effect, Selector, Value};
use crabomination::game::effects::EffectContext;
use crabomination::game::types::{GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn pod(n: usize) -> GameState {
    let mut g = multi_player_game(n);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g
}

fn ready(g: &mut GameState, def: crabomination::card::CardDefinition) -> CardId {
    let id = g.add_card_to_battlefield(0, def);
    g.clear_sickness(id);
    id
}

fn flood(g: &mut GameState) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[0].mana_pool.add(c, 6);
    }
}

fn cast(g: &mut GameState, id: CardId, target: Option<Target>) {
    flood(g);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell { card_id: id, target, additional_targets: vec![], mode: None, x_value: None })
        .expect("cast");
    drain_stack(g);
}

fn activate(g: &mut GameState, id: CardId, target: Option<Target>) -> Result<(), GameError> {
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id,
        ability_index: 0,
        target,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })?;
    drain_stack(g);
    Ok(())
}

/// CR 903.5a — Whtz's ability costs {3} less with a 200-card starting deck
/// (commanders included): free at 254 cards, unpayable with no mana at 100.
#[test]
fn whtz_ability_is_free_with_a_two_hundred_card_starting_deck() {
    let mut g = pod(2);
    g.add_card_to_library(0, catalog::island());
    let whtz = ready(&mut g, catalog::whtz_the_bibliophile());
    g.players[0].starting_deck_size = 100;
    assert!(activate(&mut g, whtz, None).is_err(), "{{3}} with nothing to pay it");
    g.players[0].starting_deck_size = 254;
    let life = g.players[0].life;
    activate(&mut g, whtz, None).expect("free at 254 cards");
    assert_eq!(g.players[0].life, life + 1);
    assert_eq!(g.players[0].hand.len(), 1);
}

/// CR 615 — Channel Harm soaks damage an opponent's source deals to you and
/// sends it at the targeted creature; damage from your own source isn't
/// soaked.
#[test]
fn channel_harm_reflects_prevented_damage_at_its_target() {
    let mut g = pod(3);
    let target = g.add_card_to_battlefield(2, catalog::hill_giant());
    let life = g.players[0].life;
    let harm = g.add_card_to_hand(0, catalog::channel_harm());
    cast(&mut g, harm, Some(Target::Permanent(target)));
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    let dctx = EffectContext::for_ability(bolt, 1, Some(Target::Player(0)));
    g.resolve_effect(&Effect::DealDamage { to: Selector::Target(0), amount: Value::Const(3) }, &dctx).unwrap();
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life, "the opponent's 3 was prevented");
    assert!(g.battlefield_find(target).is_none(), "the Hill Giant took the 3 and died");
    let own = g.add_card_to_hand(0, catalog::lightning_bolt());
    let octx = EffectContext::for_ability(own, 0, Some(Target::Player(0)));
    g.resolve_effect(&Effect::DealDamage { to: Selector::Target(0), amount: Value::Const(2) }, &octx).unwrap();
    assert_eq!(g.players[0].life, life - 2, "your own source isn't soaked");
}

/// Over the Top — each player reveals one card per nonland permanent they
/// control: permanents enter under their owner, the rest go to the
/// graveyard; a player with only lands reveals nothing.
#[test]
fn over_the_top_reveals_per_nonland_permanent() {
    let mut g = pod(3);
    ready(&mut g, catalog::grizzly_bears());
    ready(&mut g, catalog::sol_ring());
    g.add_card_to_battlefield(0, catalog::mountain());
    g.add_card_to_battlefield(1, catalog::forest());
    g.add_card_to_battlefield(2, catalog::grizzly_bears());
    for (seat, cards) in [
        (0, vec![catalog::hill_giant(), catalog::lightning_bolt(), catalog::island()]),
        (1, vec![catalog::hill_giant()]),
        (2, vec![catalog::island(), catalog::lightning_bolt()]),
    ] {
        g.players[seat].library.clear();
        for c in cards {
            g.add_card_to_library(seat, c);
        }
    }
    let ott = g.add_card_to_hand(0, catalog::over_the_top());
    cast(&mut g, ott, None);
    let on_bf = |g: &GameState, seat: usize, name: &str| {
        g.battlefield.iter().filter(|c| c.controller == seat && c.definition.name == name).count()
    };
    assert_eq!(on_bf(&g, 0, "Hill Giant"), 1);
    assert!(g.players[0].graveyard.iter().any(|c| c.definition.name == "Lightning Bolt"));
    assert_eq!(g.players[0].library.len(), 1, "two nonland permanents: two cards");
    assert_eq!(g.players[1].library.len(), 1, "only a land: nothing revealed");
    assert_eq!(on_bf(&g, 2, "Island"), 1, "a land card is a permanent card");
    assert_eq!(g.players[2].library.len(), 1);
}

/// Pattern Matcher finds a card named like a creature you control, and not
/// one named like an opponent's.
#[test]
fn pattern_matcher_searches_for_a_creature_name_you_control() {
    let mut g = pod(2);
    g.players[0].library.clear();
    g.add_card_to_library(0, catalog::hill_giant());
    g.add_card_to_library(0, catalog::grizzly_bears());
    ready(&mut g, catalog::grizzly_bears());
    g.add_card_to_battlefield(1, catalog::hill_giant());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let pm = g.add_card_to_hand(0, catalog::pattern_matcher());
    cast(&mut g, pm, None);
    let names: Vec<_> = g.players[0].hand.iter().map(|c| c.definition.name).collect();
    assert_eq!(names, vec!["Grizzly Bears"]);
}

/// Scroll of Griselbrand — the discard always, the 3 life only with a Demon.
#[test]
fn scroll_of_griselbrand_drains_only_with_a_demon() {
    for demon in [false, true] {
        let mut g = pod(2);
        g.add_card_to_hand(1, catalog::island());
        if demon {
            ready(&mut g, catalog::razorjaw_oni());
        }
        let scroll = ready(&mut g, catalog::scroll_of_griselbrand());
        flood(&mut g);
        let life = g.players[1].life;
        activate(&mut g, scroll, Some(Target::Player(1))).expect("activate");
        assert!(g.players[1].hand.is_empty());
        assert_eq!(g.players[1].life, if demon { life - 3 } else { life });
    }
}

/// Weakstone's Subjugation / Marooned — the {3} on entry taps the enchanted
/// permanent; declined, it stays untapped (the untap lock is generic).
#[test]
fn pay_to_tap_auras_tap_only_when_paid() {
    for (pay, def) in [(true, catalog::weakstones_subjugation as fn() -> _), (false, catalog::marooned)] {
        let mut g = pod(2);
        let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(pay)]));
        let aura = g.add_card_to_hand(0, def());
        cast(&mut g, aura, Some(Target::Permanent(bear)));
        assert_eq!(g.battlefield_find(bear).unwrap().tapped, pay);
        assert!(g.players[0].graveyard.is_empty() && g.players[0].hand.is_empty());
    }
}

/// Spellweaver Volute — enters on the priciest instant card in a graveyard;
/// casting a sorcery casts that card's copy free, exiles the card and moves
/// the Aura to the next instant card. With none left, it goes to the
/// graveyard.
#[test]
fn spellweaver_volute_casts_its_card_and_moves_on() {
    let mut g = pod(2);
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::island());
    }
    let opp_life = g.players[1].life;
    g.add_card_to_graveyard(1, catalog::lightning_bolt());
    let volute = g.add_card_to_hand(0, catalog::spellweaver_volute());
    cast(&mut g, volute, None);
    assert!(g.battlefield_find(volute).is_some_and(|c| c.chosen_permanent.is_some()));
    g.add_card_to_graveyard(0, catalog::shock());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let sorcery = g.add_card_to_hand(0, catalog::divination());
    cast(&mut g, sorcery, None);
    assert!(g.exile.iter().any(|c| c.definition.name == "Lightning Bolt"), "the cast card is exiled");
    assert!(g.players[1].life < opp_life, "the free Bolt copy hit the opponent");
    let shock = g.players[0].graveyard.iter().find(|c| c.definition.name == "Shock").map(|c| c.id);
    assert_eq!(g.battlefield_find(volute).and_then(|c| c.chosen_permanent), shock, "moved on to Shock");
}
