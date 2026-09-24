//! Commander: the Phantom Premonition precon (KHC, Ranar, `decks::cmdr_ranar`).

use crabomination::card::{CardId, CounterType};
use crabomination::catalog;
use crabomination::game::types::{GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn pod(n: usize) -> GameState {
    let mut g = if n == 2 { two_player_game() } else { multi_player_game(n) };
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

fn cast(g: &mut GameState, id: CardId, targets: &[Target]) {
    flood(g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: targets.first().cloned(),
        additional_targets: targets.iter().skip(1).cloned().collect(),
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(g);
}

fn named(g: &GameState, seat: usize, name: &str) -> usize {
    g.battlefield.iter().filter(|c| c.controller == seat && c.definition.name == name).count()
}

fn foretell(g: &mut GameState, id: CardId) -> Result<(), String> {
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::Foretell { card_id: id }).map(|_| ()).map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

/// Ranar: the first foretell each turn costs {0}, the second {2}; each card
/// put into exile from hand makes a Spirit (CR 603.2c — one per batch).
#[test]
fn ranar_makes_the_first_foretell_free_and_spirits_follow() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::ranar_the_ever_watchful());
    let a = g.add_card_to_hand(0, catalog::iron_verdict());
    let b = g.add_card_to_hand(0, catalog::warhorn_blast());
    foretell(&mut g, a).expect("free with an empty pool");
    assert_eq!(named(&g, 0, "Spirit"), 1);
    assert!(foretell(&mut g, b).is_err(), "the second costs {{2}}");
    g.players[0].mana_pool.add(Color::White, 2);
    foretell(&mut g, b).expect("with {2}");
    assert_eq!(named(&g, 0, "Spirit"), 2);
}

/// CR 702.143a — foretelling is a special action any time you have priority
/// during your turn, not only at sorcery speed; never on an opponent's turn.
#[test]
fn foretell_works_mid_combat_but_not_on_their_turn() {
    let mut g = pod(2);
    let a = g.add_card_to_hand(0, catalog::iron_verdict());
    g.players[0].mana_pool.add(Color::White, 4);
    g.step = TurnStep::BeginCombat;
    foretell(&mut g, a).expect("our turn, our priority");
    let b = g.add_card_to_hand(0, catalog::warhorn_blast());
    g.active_player_idx = 1;
    assert!(foretell(&mut g, b).is_err());
}

/// Hero of Bretagard counts each card exiled from hand and each permanent your
/// spells exile; five counters give it flying.
#[test]
fn hero_of_bretagard_grows_on_exile_and_flies_at_five() {
    let mut g = pod(2);
    let hero = g.add_card_to_battlefield(0, catalog::hero_of_bretagard());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let blink = g.add_card_to_hand(0, catalog::momentary_blink());
    cast(&mut g, blink, &[Target::Permanent(bear)]);
    assert_eq!(g.battlefield_find(hero).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
    for _ in 0..4 {
        let v = g.add_card_to_hand(0, catalog::iron_verdict());
        g.players[0].mana_pool.add(Color::White, 2);
        foretell(&mut g, v).expect("foretell");
    }
    let h = g.battlefield_find(hero).unwrap();
    assert_eq!(h.counter_count(CounterType::PlusOnePlusOne), 5);
    let cp = g.computed_permanent(hero).unwrap();
    assert!(cp.keywords().contains(&crabomination::card::Keyword::Flying));
}

/// Ethereal Valkyrie foretells a card that has no foretell of its own; it is
/// cast from exile on a later turn for its mana cost less {2}.
#[test]
fn ethereal_valkyrie_grants_a_foretell_cost() {
    let mut g = pod(2);
    g.add_card_to_library(0, catalog::island());
    let wurm = g.add_card_to_hand(0, catalog::craw_wurm());
    let v = g.add_card_to_hand(0, catalog::ethereal_valkyrie());
    cast(&mut g, v, &[]);
    let exiled = g.exile.iter().find(|c| c.id == wurm).expect("foretold");
    assert!(exiled.face_down);
    g.players[0].mana_pool.empty();
    g.priority.player_with_priority = 0;
    let cast_it = |g: &mut GameState| {
        g.perform_action(GameAction::CastForetold {
            card_id: wurm,
            target: None,
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
    };
    g.players[0].mana_pool.add(Color::Green, 4);
    assert!(cast_it(&mut g).is_err(), "not on the turn it was foretold");
    g.foretold_this_turn.clear();
    cast_it(&mut g).expect("{2}{G}{G}: Craw Wurm's {4}{G}{G} less {2}");
    drain_stack(&mut g);
    assert_eq!(named(&g, 0, "Craw Wurm"), 1);
    assert_eq!(g.players[0].mana_pool.total(), 0);
}

/// Niko Defies Destiny I: 2 life per foretold card you own in exile.
#[test]
fn niko_chapter_one_counts_foretold_cards() {
    let mut g = pod(2);
    for _ in 0..2 {
        let c = g.add_card_to_hand(0, catalog::iron_verdict());
        g.players[0].mana_pool.add(Color::White, 2);
        foretell(&mut g, c).expect("foretell");
    }
    let life = g.players[0].life;
    let niko = g.add_card_to_hand(0, catalog::niko_defies_destiny());
    cast(&mut g, niko, &[]);
    assert_eq!(g.players[0].life, life + 4);
}

/// Cosmic Intervention: your creature that would die is exiled instead and
/// returns at the next end step.
#[test]
fn cosmic_intervention_returns_the_fallen() {
    let mut g = pod(2);
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let ci = g.add_card_to_hand(0, catalog::cosmic_intervention());
    cast(&mut g, ci, &[]);
    let murder = g.add_card_to_hand(0, catalog::murder());
    cast(&mut g, murder, &[Target::Permanent(bear)]);
    assert!(g.exile.iter().any(|c| c.id == bear), "exiled instead of dying");
    while g.step != TurnStep::End {
        let _ = g.advance_step(Vec::new());
        drain_stack(&mut g);
    }
    drain_stack(&mut g);
    assert_eq!(named(&g, 0, "Grizzly Bears"), 1);
}

/// Arcane Artisan: the player exiles a creature card from hand for a token
/// copy; when Artisan leaves, its tokens go at the next end step.
#[test]
fn arcane_artisan_copies_then_its_tokens_leave() {
    let mut g = pod(2);
    let art = g.add_card_to_battlefield(0, catalog::arcane_artisan());
    g.clear_sickness(art);
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_hand(0, catalog::craw_wurm());
    flood(&mut g, 0);
    g.perform_action(GameAction::ActivateAbility {
        card_id: art,
        ability_index: 0,
        target: Some(Target::Player(0)),
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("activate");
    drain_stack(&mut g);
    assert_eq!(named(&g, 0, "Craw Wurm"), 1, "a token copy");
    let murder = g.add_card_to_hand(0, catalog::murder());
    cast(&mut g, murder, &[Target::Permanent(art)]);
    while g.step != TurnStep::End {
        let _ = g.advance_step(Vec::new());
        drain_stack(&mut g);
    }
    drain_stack(&mut g);
    assert_eq!(named(&g, 0, "Craw Wurm"), 0);
}

/// Spectral Deluge: toughness at most your Islands goes back.
#[test]
fn spectral_deluge_counts_islands() {
    let mut g = pod(2);
    for _ in 0..2 {
        g.add_card_to_battlefield(0, catalog::island());
    }
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let wurm = g.add_card_to_battlefield(1, catalog::craw_wurm());
    let sd = g.add_card_to_hand(0, catalog::spectral_deluge());
    cast(&mut g, sd, &[]);
    assert!(g.battlefield_find(bear).is_none());
    assert!(g.battlefield_find(wurm).is_some());
}

/// Tales of the Ancestors: everyone draws up to the biggest hand.
#[test]
fn tales_of_the_ancestors_levels_hands() {
    let mut g = pod(3);
    for s in 0..3 {
        for _ in 0..5 {
            g.add_card_to_library(s, catalog::island());
        }
    }
    for _ in 0..4 {
        g.add_card_to_hand(1, catalog::island());
    }
    g.add_card_to_hand(2, catalog::island());
    let t = g.add_card_to_hand(0, catalog::tales_of_the_ancestors());
    cast(&mut g, t, &[]);
    assert_eq!([g.players[0].hand.len(), g.players[1].hand.len(), g.players[2].hand.len()], [4, 4, 4]);
}

/// Thunderclap Wyvern pumps the other flyers only.
#[test]
fn thunderclap_wyvern_lifts_flyers() {
    let mut g = pod(2);
    let wy = g.add_card_to_battlefield(0, catalog::thunderclap_wyvern());
    let sphinx = g.add_card_to_battlefield(0, catalog::inspired_sphinx());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let pt = |g: &GameState, id| {
        let c = g.computed_permanent(id).unwrap();
        (c.power, c.toughness)
    };
    assert_eq!(pt(&g, sphinx), (6, 6));
    assert_eq!(pt(&g, bear), (2, 2));
    assert_eq!(pt(&g, wy), (2, 3));
}

/// Replicating Ring: the eighth night counter makes eight Replicated Rings.
#[test]
fn replicating_ring_replicates_on_the_eighth_night() {
    let mut g = pod(2);
    let ring = g.add_card_to_battlefield(0, catalog::replicating_ring());
    g.battlefield_find_mut(ring).unwrap().add_counters(CounterType::Night, 7);
    for s in 0..2 {
        g.add_card_to_library(s, catalog::island());
    }
    g.active_player_idx = 0;
    g.step = TurnStep::Untap;
    g.priority.player_with_priority = 0;
    while g.step != TurnStep::Draw {
        let _ = g.advance_step(Vec::new());
        drain_stack(&mut g);
    }
    assert_eq!(named(&g, 0, "Replicated Ring"), 8);
    assert_eq!(g.battlefield_find(ring).unwrap().counter_count(CounterType::Night), 0);
}

/// Stoic Farmer: behind on lands, the Plains comes in tapped; otherwise hand.
#[test]
fn stoic_farmer_catches_up() {
    for (their_lands, on_field) in [(2, true), (0, false)] {
        let mut g = pod(2);
        for _ in 0..their_lands {
            g.add_card_to_battlefield(1, catalog::plains());
        }
        g.add_card_to_library(0, catalog::plains());
        let f = g.add_card_to_hand(0, catalog::stoic_farmer());
        cast(&mut g, f, &[]);
        assert_eq!(named(&g, 0, "Plains") == 1, on_field);
        assert_eq!(g.players[0].hand.iter().any(|c| c.definition.name == "Plains"), !on_field);
    }
}
